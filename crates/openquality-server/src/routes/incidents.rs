use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

use crate::auth_extractor::AuthenticatedUser;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListIncidentsQuery {
    status: Option<String>,
    monitor_id: Option<String>,
}

#[derive(Deserialize)]
pub struct SnoozeRequest {
    until: DateTime<Utc>,
}

pub async fn list_incidents(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListIncidentsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mgr = state.alert_manager.read().await;
    let incidents = mgr.list(query.status.as_deref()).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    let incidents = if let Some(mid) = &query.monitor_id {
        incidents
            .into_iter()
            .filter(|i| i.monitor_id == *mid)
            .collect()
    } else {
        incidents
    };

    Ok(Json(json!(incidents)))
}

pub async fn get_incident(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mgr = state.alert_manager.read().await;
    let incident = mgr
        .get(id)
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
                Json(json!({"error": "incident not found"})),
            )
        })?;
    Ok(Json(json!(incident)))
}

pub async fn resolve_incident(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mgr = state.alert_manager.read().await;
    let incident = mgr
        .resolve(id)
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
                Json(json!({"error": "incident not found"})),
            )
        })?;
    Ok(Json(json!(incident)))
}

pub async fn acknowledge_incident(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mgr = state.alert_manager.read().await;
    let incident = mgr
        .acknowledge(id, Some(_auth.claims.sub))
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
                Json(json!({"error": "incident not found"})),
            )
        })?;
    Ok(Json(json!(incident)))
}

pub async fn snooze_incident(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<SnoozeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mgr = state.alert_manager.read().await;
    let incident = mgr
        .snooze(id, req.until)
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
                Json(json!({"error": "incident not found"})),
            )
        })?;
    Ok(Json(json!(incident)))
}
