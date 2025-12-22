use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

#[derive(Template)]
#[template(path = "error.html")]
pub struct SecretErrorTemplate<'a> {
    pub message: &'a str,
}

impl IntoResponse for SecretErrorTemplate<'_> {
    fn into_response(self) -> Response {
        match self.render() {
            Ok(html) => (StatusCode::UNAUTHORIZED, Html(html)).into_response(),
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Template rendering failed",
            )
                .into_response(),
        }
    }
}

pub fn render_secret_error_page(message: &'static str) -> SecretErrorTemplate<'static> {
    SecretErrorTemplate { message }
}
