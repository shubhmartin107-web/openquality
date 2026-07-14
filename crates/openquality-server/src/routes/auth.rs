use axum::{Json, extract::State, http::StatusCode};
use openquality_auth::Role;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use openquality_auth::{hash_password, verify_password};

use crate::auth_extractor::AuthenticatedUser;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct RegisterRequest {
    email: String,
    password: String,
    name: String,
    workspace_name: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    token: String,
    user_id: Uuid,
    workspace_id: Uuid,
    role: String,
}

#[derive(Deserialize)]
pub struct CreateApiKeyRequest {
    label: String,
}

#[derive(Serialize)]
pub struct ApiKeyResponse {
    id: Uuid,
    key: String,
    label: String,
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let ws = state
        .store
        .create_workspace(
            &req.workspace_name,
            &req.workspace_name.to_lowercase().replace(' ', "-"),
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;

    let password_hash = hash_password(&req.password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    let user = state
        .store
        .create_user_with_password(ws.id, &req.email, &req.name, "Owner", &password_hash)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;

    let role = Role::Owner;
    let token = state
        .jwt_manager
        .create_token(user.id, ws.id, &req.email, &role)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;

    Ok(Json(json!(AuthResponse {
        token,
        user_id: user.id,
        workspace_id: ws.id,
        role: role.as_str().to_string(),
    })))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let user = state
        .store
        .get_user_by_email(&req.email)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "invalid email or password"})),
            )
        })?;

    let valid = verify_password(&req.password, &user.password_hash).unwrap_or(false);
    if !valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "invalid email or password"})),
        ));
    }

    let role = Role::from_str(&user.role).unwrap_or(Role::Viewer);
    let token = state
        .jwt_manager
        .create_token(user.id, user.workspace_id, &user.email, &role)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;

    Ok(Json(json!(AuthResponse {
        token,
        user_id: user.id,
        workspace_id: user.workspace_id,
        role: role.as_str().to_string(),
    })))
}

pub async fn refresh_token(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let token = state
        .jwt_manager
        .refresh_token(&_auth.claims)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;

    Ok(Json(json!({"token": token})))
}

pub async fn list_api_keys(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let keys = state
        .store
        .list_api_keys(_auth.claims.sub)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok(Json(json!(keys)))
}

pub async fn create_api_key(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    use openquality_auth::generate_api_key;
    let pair = generate_api_key();
    let record = state
        .store
        .store_api_key(_auth.claims.sub, &pair.key_hash, &req.label)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;

    Ok(Json(json!(ApiKeyResponse {
        id: record.id,
        key: pair.raw_key,
        label: record.label,
    })))
}

pub async fn revoke_api_key(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    state.store.revoke_api_key(id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
    })?;
    Ok(Json(json!({"status": "revoked"})))
}
