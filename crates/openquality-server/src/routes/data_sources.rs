use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

use crate::auth_extractor::AuthenticatedUser;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct CreateDataSourceRequest {
    name: String,
    source_type: String,
    config: serde_json::Value,
}

pub async fn create_data_source(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    Json(req): Json<CreateDataSourceRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let ds = state
        .store
        .create_data_source(ws_id, &req.name, &req.source_type, req.config)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok(Json(json!(ds)))
}

pub async fn list_data_sources(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let sources = state.store.list_data_sources(ws_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
    })?;
    Ok(Json(json!(sources)))
}

pub async fn get_data_source(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let ds = state
        .store
        .get_data_source(id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "data source not found"})),
            )
        })?;
    Ok(Json(json!(ds)))
}

pub async fn delete_data_source(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    state.store.delete_data_source(id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
    })?;
    Ok(Json(json!({"status": "deleted"})))
}
