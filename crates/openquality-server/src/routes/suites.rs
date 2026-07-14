use axum::{
    Json,
    extract::{Path, State},
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
pub struct CreateSuiteRequest {
    name: String,
}

#[derive(Deserialize)]
pub struct CreateExpectationRequest {
    expectation_type: String,
    column: Option<String>,
}

pub async fn create_suite(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    Json(req): Json<CreateSuiteRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let suite = state
        .store
        .create_suite(ws_id, &req.name)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok(Json(json!(suite)))
}

pub async fn list_suites(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let suites = state.store.list_suites(ws_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
    })?;
    Ok(Json(json!(suites)))
}

pub async fn get_suite(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let suite = state
        .store
        .get_suite(id)
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
                Json(json!({"error": "suite not found"})),
            )
        })?;
    Ok(Json(json!(suite)))
}

pub async fn delete_suite(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    state.store.delete_suite(id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
    })?;
    Ok(Json(json!({"status": "deleted"})))
}

pub async fn run_suite(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let suite_record = state
        .store
        .get_suite(id)
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
                Json(json!({"error": "suite not found"})),
            )
        })?;

    let expectations = state.store.list_expectations(id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    let mut suite = ExpectationSuite::new(&suite_record.name);
    for ex in &expectations {
        suite.add(Expectation::new(
            ex.expectation_type.clone(),
            ex.column.as_deref(),
        ));
    }

    let _run = state
        .store
        .create_suite_run(id, "running")
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;

    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(
            json!({"error": "suite execution not yet implemented - connect to data source connector"}),
        ),
    ))
}

pub async fn list_suite_runs(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Path(suite_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let runs = state.store.list_suite_runs(suite_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
    })?;
    Ok(Json(json!(runs)))
}

pub async fn list_expectations(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Path(suite_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let exps = state.store.list_expectations(suite_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
    })?;
    Ok(Json(json!(exps)))
}

pub async fn create_expectation(
    _auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Path(suite_id): Path<Uuid>,
    Json(req): Json<CreateExpectationRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let exp_type = parse_expectation_type(&req.expectation_type).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("unknown expectation type: {}", req.expectation_type)})),
        )
    })?;
    let exp = state
        .store
        .create_expectation(suite_id, &exp_type, req.column.as_deref())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok(Json(json!(exp)))
}

fn parse_expectation_type(s: &str) -> Option<ExpectationType> {
    match s.to_lowercase().as_str() {
        "not_null" => Some(ExpectationType::NotNull),
        "unique" => Some(ExpectationType::Unique),
        "match_regex" => Some(ExpectationType::MatchRegex(String::new())),
        "not_match_regex" => Some(ExpectationType::NotMatchRegex(String::new())),
        "row_count_between" => Some(ExpectationType::RowCountBetween(0, 0)),
        "column_mean_between" => Some(ExpectationType::ColumnMeanBetween(0.0, 0.0)),
        "column_stddev_between" => Some(ExpectationType::ColumnStddevBetween(0.0, 0.0)),
        "column_min_between" => Some(ExpectationType::ColumnMinBetween(0.0, 0.0)),
        "column_max_between" => Some(ExpectationType::ColumnMaxBetween(0.0, 0.0)),
        "values_in_set" => Some(ExpectationType::ColumnValuesToBeInSet(vec![])),
        "kl_divergence" => Some(ExpectationType::ColumnKLDivergenceLessThan(0.0)),
        "quantile_between" => Some(ExpectationType::ColumnQuantileBetween(0.0, 0.0, 0.0)),
        "distinct_values_equal_set" => Some(ExpectationType::DistinctValuesEqualSet(vec![])),
        "distinct_values_contained_in_set" => {
            Some(ExpectationType::DistinctValuesContainedInSet(vec![]))
        }
        "table_columns_match" => Some(ExpectationType::TableColumnsMatchOrderedList(vec![])),
        _ => None,
    }
}
