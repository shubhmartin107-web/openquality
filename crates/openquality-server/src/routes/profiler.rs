use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

use crate::auth_extractor::AuthenticatedUser;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ProfileRequest {
    data_source_id: Uuid,
    _table_name: String,
}

#[derive(Deserialize)]
pub struct SuggestRequest {
    data_source_id: Uuid,
}

pub async fn profile_table(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<ProfileRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let _ds = state
        .store
        .get_data_source(req.data_source_id)
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

    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "profiling not yet implemented - connect to data source connector"})),
    ))
}

pub async fn suggest_expectations(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<SuggestRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let _ds = state
        .store
        .get_data_source(req.data_source_id)
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

    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(
            json!({"error": "suggestions not yet implemented - connect to data source and fetch data"}),
        ),
    ))
}
