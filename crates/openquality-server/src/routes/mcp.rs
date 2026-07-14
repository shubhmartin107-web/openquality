use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use openquality_core::store::Store;
use openquality_mcp::tools;

use crate::auth_extractor::AuthenticatedUser;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct McpRequest {
    tool: String,
    params: Value,
}

pub async fn execute_mcp(
    auth: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<McpRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let store: &dyn Store = &*state.store;
    let ws_id = auth.claims.workspace_id;
    let result = match req.tool.as_str() {
        "run_suite" => tools::run_suite(store, ws_id, req.params).await,
        "validate_data" => tools::validate_data(store, ws_id, req.params).await,
        "profile_data" => tools::profile_data(store, ws_id, req.params).await,
        "suggest_expectations" => tools::suggest_expectations(store, ws_id, req.params).await,
        "list_monitors" => tools::list_monitors(store, ws_id, req.params).await,
        "list_incidents" => tools::list_incidents(store, ws_id, req.params).await,
        "resolve_incident" => tools::resolve_incident(store, ws_id, req.params).await,
        "health" => tools::health(store, ws_id, req.params).await,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": format!("unknown tool: {}", req.tool)})),
            ));
        }
    };

    match result {
        Ok(val) => Ok(Json(val)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}
