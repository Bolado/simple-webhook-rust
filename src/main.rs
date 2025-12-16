use axum::{
    Json, Router,
    extract::{Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    env,
    fmt::Debug,
    sync::{Arc, Mutex},
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // shared state for storing webhooks and the expected secret
    let app_state = AppState {
        webhooks: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
        secret: std::env::var("WEBHOOK_SECRET").unwrap_or_else(|_| "DEFAULT_KEY".into()),
    };

    let secret = app_state.secret.clone();

    // same endpoint for both GET and POST
    // GET to check stored webhooks, POST to receive new webhooks
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/", post(webhook_handler))
        .with_state(app_state);

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string()); // default port 3000
    let addr = format!("0.0.0.0:{}", port); // bind to all interfaces
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap(); // create the listener

    // display a ferris message
    ferris_says::say(
        format!("Server running on :{}", port).as_str(),
        50,
        &mut std::io::stdout(),
    )
    .unwrap();

    // display the access URL with the expected secret
    println!(
        "Access the received webhooks through http://localhost:{}/?secret={}",
        port, secret
    );

    axum::serve(listener, app).await.unwrap();
}

// root_handler checks the secret and returns every stored webhook
async fn root_handler(
    State(AppState {
        webhooks,
        secret: expected_secret,
    }): State<AppState>,
    Query(params): Query<SecretQuery>,
) -> impl IntoResponse {
    // if no secret provided
    let Some(provided_secret) = params.secret else {
        return (
            StatusCode::UNAUTHORIZED,
            [(header::CONTENT_TYPE, "text/plain")],
            "No secret provided.".to_string(),
        );
    };

    // if wrong secret provided
    if provided_secret != expected_secret {
        return (
            StatusCode::UNAUTHORIZED,
            [(header::CONTENT_TYPE, "text/plain")],
            "Wrong secret.".to_string(),
        );
    }

    // serialize every stored webhook as pretty JSON
    let document_body = {
        let webhooks = webhooks.lock().unwrap();
        webhooks
            .iter()
            .map(|webhook| serde_json::to_string_pretty(webhook).unwrap())
            .collect::<Vec<String>>()
            .join("\n")
    };

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        document_body,
    )
}

// webhook_handler stores the received webhook payload
async fn webhook_handler(
    State(AppState { webhooks, .. }): State<AppState>,
    Json(payload): Json<WebhookPayload>,
) -> StatusCode {
    println!("Received a webhook!");

    let mut webhooks = webhooks.lock().unwrap();

    // maintain a maximum of 100 stored webhooks
    if webhooks.len() >= 100 {
        webhooks.pop_front();
    }

    // store the new webhook
    webhooks.push_back(payload);
    StatusCode::OK
}

fn current_timestamp() -> String {
    let now = chrono::Utc::now();
    now.to_rfc3339().to_string()
}

// WebhookPayload represents the structure of the received webhook payload
// Includes the default timestamp if not provided
// And squishes all other fields which may or may not be present
#[derive(Debug, Serialize, Deserialize)]
struct WebhookPayload {
    #[serde(default = "current_timestamp")]
    timestamp: String,
    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}

// SecretQuery represents the expected query parameter for secret
// We set it as optional to handle the case where it's not provided
#[derive(Deserialize)]
struct SecretQuery {
    secret: Option<String>,
}

// AppState holds the shared state for the application
#[derive(Clone)]
struct AppState {
    webhooks: Arc<Mutex<VecDeque<WebhookPayload>>>,
    secret: String,
}
