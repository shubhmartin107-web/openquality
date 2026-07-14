use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

use openquality_core::types::*;

use crate::auth_extractor::AuthenticatedUser;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct CreateMonitorRequest {
    name: String,
    monitor_type: String,
    table_name: String,
    config: Option<serde_json::Value>,
    schedule_cron: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateMonitorRequest {
    enabled: Option<bool>,
    schedule_cron: Option<String>,
    config: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct MonitorHistoryQuery {
    limit: Option<i64>,
}

pub async fn create_monitor(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    Json(req): Json<CreateMonitorRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mon_type = parse_monitor_type(&req.monitor_type).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("unknown monitor type: {}", req.monitor_type)})),
        )
    })?;
    let config = req.config.unwrap_or_default();
    let monitor = state
        .store
        .create_monitor(ws_id, &req.name, &mon_type, &req.table_name, config)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;

    if let Some(cron) = req.schedule_cron {
        state
            .store
            .update_monitor(monitor.id, json!({"schedule_cron": cron}))
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()})),
                )
            })?;
    }

    Ok(Json(json!(monitor)))
}

pub async fn list_monitors(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let monitors = state.store.list_monitors(ws_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
    })?;
    Ok(Json(json!(monitors)))
}

pub async fn get_monitor(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let monitor = state
        .store
        .get_monitor(id)
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
                Json(json!({"error": "monitor not found"})),
            )
        })?;
    Ok(Json(json!(monitor)))
}

pub async fn update_monitor(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateMonitorRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let config = req.config.unwrap_or_default();
    state.store.update_monitor(id, config).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    if let Some(enabled) = req.enabled {
        state
            .store
            .update_monitor(id, json!({"enabled": enabled}))
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()})),
                )
            })?;
    }
    if let Some(cron) = req.schedule_cron {
        state
            .store
            .update_monitor(id, json!({"schedule_cron": cron}))
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()})),
                )
            })?;
    }

    let monitor = state
        .store
        .get_monitor(id)
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
                Json(json!({"error": "monitor not found"})),
            )
        })?;
    Ok(Json(json!(monitor)))
}

pub async fn delete_monitor(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    state.store.delete_monitor(id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
    })?;
    Ok(Json(json!({"status": "deleted"})))
}

pub async fn run_monitor(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let monitor = state
        .store
        .get_monitor(id)
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
                Json(json!({"error": "monitor not found"})),
            )
        })?;

    let result = MonitorResult {
        monitor_id: monitor.id.to_string(),
        monitor_type: monitor.monitor_type.clone(),
        table_name: monitor.table_name.clone(),
        alert: false,
        severity: Severity::Info,
        score: 0.0,
        threshold: 0.0,
        message: format!("Monitor '{}' executed (placeholder)", monitor.name),
        details: std::collections::HashMap::new(),
        timestamp: chrono::Utc::now(),
    };

    state
        .store
        .append_monitor_history(id, result.score, result.threshold)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;

    Ok(Json(json!(result)))
}

pub async fn get_monitor_history(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<MonitorHistoryQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let limit = query.limit.unwrap_or(100);
    let history = state
        .store
        .get_monitor_history(id, limit)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok(Json(json!(history)))
}

fn parse_monitor_type(s: &str) -> Option<MonitorType> {
    match s.to_lowercase().as_str() {
        "freshness" => Some(MonitorType::Freshness {
            max_age_seconds: 86400,
        }),
        "volume" => Some(MonitorType::Volume {
            baseline_period_seconds: 86400,
        }),
        "schema" => Some(MonitorType::Schema),
        "distribution" => Some(MonitorType::Distribution {
            metric: DistributionMetric::KSTest,
            column: None,
        }),
        "correlation" | "corr" => Some(MonitorType::Correlation {
            column_x: String::new(),
            column_y: String::new(),
            method: CorrelationMethod::Pearson,
            threshold: 0.8,
        }),
        "uniqueness" | "unique" => Some(MonitorType::Uniqueness {
            columns: vec![],
            threshold_ratio: 0.01,
        }),
        "referential" | "referential_integrity" | "ref" => {
            Some(MonitorType::ReferentialIntegrity {
                source_column: String::new(),
                target_table: String::new(),
                target_column: String::new(),
            })
        }
        "custom_sql" | "customsql" => Some(MonitorType::CustomSQL {
            query: String::new(),
            expected: None,
            comparison: ComparisonOp::EqualTo,
        }),
        "ml_drift" | "mldrift" => Some(MonitorType::MLDrift {
            model_name: String::new(),
            metric: MLDriftMetric::PredictionDrift,
            threshold: 0.1,
        }),
        "cost" => Some(MonitorType::Cost {
            resource_type: CostResourceType::QueryCost,
            budget: 1000.0,
        }),
        _ => None,
    }
}
