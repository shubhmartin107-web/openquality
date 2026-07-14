use axum::{
    Json,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use openquality_auth::Claims;
use serde_json::json;
use std::sync::Arc;

use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub claims: Claims,
}

pub struct AuthError;

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "missing or invalid authorization"})),
        )
            .into_response()
    }
}

impl FromRequestParts<Arc<AppState>> for AuthenticatedUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let token = auth_header.strip_prefix("Bearer ").unwrap_or("");

        if token.is_empty() {
            return Err(AuthError);
        }

        match state.jwt_manager.validate_token(token) {
            Ok(claims) => Ok(AuthenticatedUser { claims }),
            Err(_) => Err(AuthError),
        }
    }
}
