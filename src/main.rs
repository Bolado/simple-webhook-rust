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
    sync::{Arc, Mutex},
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // VecDeque to make it easy to pop from the front when we reach capacity
    let webhooks: Arc<Mutex<VecDeque<WebhookPayload>>> =
        Arc::new(Mutex::new(VecDeque::with_capacity(100)));

    // same endpoint for both GET and POST
    // GET to check stored webhooks, POST to receive new webhooks
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/", post(webhook_handler))
        .with_state(webhooks);

    // get optional port
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    // display a ferris message
    let message = format!("Server running on :{}", port);
    ferris_says::say(message.as_str(), 50, &mut std::io::stdout()).unwrap();

    // display the expected secret
    let expected_secret = env::var("WEBHOOK_SECRET").unwrap_or_else(|_| "DEFAULT_KEY".to_string());
    println!(
        "Access the received webhooks through http://yourip:{}/?secret={}",
        port, expected_secret
    );

    axum::serve(listener, app).await.unwrap();
}

// root_handler checks the secret and returns the every stored webhooks
async fn root_handler(
    State(webhooks): State<Arc<Mutex<VecDeque<WebhookPayload>>>>,
    Query(params): Query<SecretQuery>,
) -> impl IntoResponse {
    let expected_secret = env::var("WEBHOOK_SECRET").unwrap_or_else(|_| "DEFAULT_KEY".to_string());

    let Some(provided_secret) = params.secret else {
        return (
            StatusCode::UNAUTHORIZED,
            [(header::CONTENT_TYPE, "text/plain")],
            "No secret provided.".to_string(),
        );
    };

    if provided_secret != expected_secret {
        return (
            StatusCode::UNAUTHORIZED,
            [(header::CONTENT_TYPE, "text/plain")],
            "Wrong secret.".to_string(),
        );
    }

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

async fn webhook_handler(
    State(webhooks): State<Arc<Mutex<VecDeque<WebhookPayload>>>>,
    Json(payload): Json<WebhookPayload>,
) -> StatusCode {
    println!("Received webhook");

    let mut webhooks = webhooks.lock().unwrap();

    if webhooks.len() >= 100 {
        webhooks.pop_front();
    }

    webhooks.push_back(payload);
    StatusCode::OK
}

#[derive(Debug, Serialize, Deserialize)]
struct WebhookPayload {
    r#type: String,
    timestamp: String,
    data: serde_json::Value,
}

#[derive(Deserialize)]
struct SecretQuery {
    secret: Option<String>,
}
