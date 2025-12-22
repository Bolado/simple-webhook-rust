use axum::{
    Json, Router,
    extract::{Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    env,
    fmt::Debug,
    sync::{Arc, Mutex},
};

// WebhookPayload represents the structure of the received webhook payload
// Includes the default timestamp if not provided
// And squishes all other fields which may or may not be present
#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // use env WEBHOOK_SECRET if set & non-empty, otherwise generate random
    let secret = match env::var("WEBHOOK_SECRET") {
        Ok(s) if !s.is_empty() => s,
        _ => {
            let generated = generate_secret(32);
            println!("WEBHOOK_SECRET not set, generated secret: {}", generated);
            generated
        }
    };

    // shared state for storing webhooks and the expected secret
    let app_state = AppState {
        webhooks: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
        secret: secret.clone(),
    };

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
        // Grab the lock once
        let webhooks = webhooks.lock().unwrap();

        // Clone the webhooks into a Vec to avoid holding the lock during serialization
        let vec: Vec<WebhookPayload> = webhooks.iter().cloned().collect();

        // Serialize the Vec to pretty JSON
        serde_json::to_string_pretty(&vec).unwrap()
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
    chrono::Utc::now().to_rfc3339()
}

fn generate_secret(len: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
