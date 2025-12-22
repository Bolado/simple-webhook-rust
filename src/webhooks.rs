use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

#[derive(Template)]
#[template(path = "webhooks.html")]
pub struct WebhooksTemplate {
    pub endpoint: String,
    pub webhooks: Vec<WebhookDisplay>,
    pub webhooks_count: usize,
}

pub struct WebhookDisplay {
    pub timestamp: String,
    pub json: String,
}

impl IntoResponse for WebhooksTemplate {
    fn into_response(self) -> Response {
        match self.render() {
            Ok(html) => Html(html).into_response(),
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Template rendering failed",
            )
                .into_response(),
        }
    }
}
