use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use openquality_integrations::airflow::AirflowWebhookPayload;
use openquality_integrations::dbt::DbtManifest;
use openquality_integrations::ge::GeSuite;
use openquality_integrations::lineage::LineageParser;

use crate::auth_extractor::AuthenticatedUser;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ParseManifestRequest {
    manifest_json: Value,
}

pub async fn dbt_parse_manifest(
    _auth: AuthenticatedUser,
    _state: State<Arc<AppState>>,
    Json(req): Json<ParseManifestRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let json_str = serde_json::to_string(&req.manifest_json).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    let manifest = DbtManifest::from_json(&json_str).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    let models = manifest.list_models();
    let sources = manifest.list_sources();
    let edges = manifest.extract_lineage_edges();

    Ok(Json(json!({
        "models": models,
        "sources": sources,
        "lineage_edges": edges,
        "model_count": models.len(),
        "source_count": sources.len(),
    })))
}

pub async fn airflow_webhook(
    _auth: AuthenticatedUser,
    _state: State<Arc<AppState>>,
    Json(payload): Json<AirflowWebhookPayload>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let parsed = payload.parse().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    Ok(Json(json!({
        "event": format!("{:?}", parsed.event),
        "dag_id": parsed.dag_id,
        "run_id": parsed.run_id,
        "is_terminal": parsed.is_terminal(),
        "is_success": parsed.is_success(),
        "is_failure": parsed.is_failure(),
        "affected_tables": parsed.affected_tables(),
        "duration_seconds": parsed.duration_seconds,
        "task_id": parsed.task_id,
        "execution_date": parsed.execution_date,
        "dag_run_url": parsed.dag_run_url,
    })))
}

#[derive(Deserialize)]
pub struct GeTranslateRequest {
    suite_json: Value,
}

pub async fn ge_translate(
    _auth: AuthenticatedUser,
    _state: State<Arc<AppState>>,
    Json(req): Json<GeTranslateRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let json_str = serde_json::to_string(&req.suite_json).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    let suite = GeSuite::from_json(&json_str).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    let translated = suite.translate();

    let expectations: Vec<Value> = translated
        .into_iter()
        .map(|t| {
            json!({
                "expectation_type": format!("{:?}", t.expectation_type),
                "column": t.column,
                "kwargs": t.kwargs,
            })
        })
        .collect();

    Ok(Json(json!({
        "suite_name": suite.name,
        "expectations": expectations,
        "count": expectations.len(),
    })))
}

#[derive(Deserialize)]
pub struct ParseSqlRequest {
    sql: String,
}

pub async fn lineage_parse_sql(
    _auth: AuthenticatedUser,
    _state: State<Arc<AppState>>,
    Json(req): Json<ParseSqlRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let parser = LineageParser::new();
    let stmt = parser.parse_sql(&req.sql);

    Ok(Json(json!({
        "target_table": stmt.target_table,
        "source_tables": stmt.source_tables,
        "column_mappings": stmt.column_mappings,
    })))
}

#[derive(Deserialize)]
pub struct LineageBatchRequest {
    statements: Vec<String>,
}

pub async fn lineage_build_graph(
    _auth: AuthenticatedUser,
    _state: State<Arc<AppState>>,
    Json(req): Json<LineageBatchRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let parser = LineageParser::new();
    let stmts: Vec<_> = req.statements.iter().map(|s| parser.parse_sql(s)).collect();
    let tables = parser.build_table_lineage(&stmts);

    Ok(Json(json!({
        "tables": tables,
    })))
}
