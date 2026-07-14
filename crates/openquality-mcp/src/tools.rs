use uuid::Uuid;

use openquality_core::store::{IncidentUpdate, Store};
use serde_json::{Value, json};

pub struct McpTools;

impl McpTools {
    pub fn list_tools() -> Vec<McpTool> {
        vec![
            McpTool {
                name: "run_suite".into(),
                description: "Run an expectation suite against a data source".into(),
            },
            McpTool {
                name: "validate_data".into(),
                description: "Validate data against expectations".into(),
            },
            McpTool {
                name: "profile_data".into(),
                description: "Profile columns of a data source".into(),
            },
            McpTool {
                name: "suggest_expectations".into(),
                description: "Suggest expectations based on column profiles".into(),
            },
            McpTool {
                name: "list_monitors".into(),
                description: "List all registered monitors".into(),
            },
            McpTool {
                name: "list_incidents".into(),
                description: "Query open/resolved incidents".into(),
            },
            McpTool {
                name: "resolve_incident".into(),
                description: "Mark incident as resolved".into(),
            },
            McpTool {
                name: "health".into(),
                description: "Check server health".into(),
            },
        ]
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
}

pub type McpResult = Result<Value, String>;

pub async fn run_suite(_store: &dyn Store, _workspace_id: Uuid, _params: Value) -> McpResult {
    Ok(
        json!({"status": "not_implemented", "message": "Suite execution via MCP requires a connected data source connector"}),
    )
}

pub async fn validate_data(_store: &dyn Store, _workspace_id: Uuid, _params: Value) -> McpResult {
    Ok(
        json!({"status": "not_implemented", "message": "Data validation via MCP requires a connected data source connector"}),
    )
}

pub async fn profile_data(_store: &dyn Store, _workspace_id: Uuid, _params: Value) -> McpResult {
    Ok(
        json!({"status": "not_implemented", "message": "Profiling via MCP requires a connected data source connector"}),
    )
}

pub async fn suggest_expectations(
    _store: &dyn Store,
    _workspace_id: Uuid,
    _params: Value,
) -> McpResult {
    Ok(
        json!({"status": "not_implemented", "message": "Suggestion via MCP requires a connected data source connector"}),
    )
}

pub async fn list_monitors(store: &dyn Store, workspace_id: Uuid, _params: Value) -> McpResult {
    match store.list_monitors(workspace_id).await {
        Ok(monitors) => Ok(json!({"monitors": monitors})),
        Err(e) => Err(e.to_string()),
    }
}

pub async fn list_incidents(store: &dyn Store, workspace_id: Uuid, params: Value) -> McpResult {
    let status = params.get("status").and_then(|v| v.as_str());
    match store.list_incidents(workspace_id, status).await {
        Ok(incidents) => Ok(json!({"incidents": incidents})),
        Err(e) => Err(e.to_string()),
    }
}

pub async fn resolve_incident(store: &dyn Store, _workspace_id: Uuid, params: Value) -> McpResult {
    let id_str = params
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing 'id' parameter (UUID string)".to_string())?;
    let id = Uuid::parse_str(id_str).map_err(|e| format!("invalid UUID: {}", e))?;
    let updates = IncidentUpdate {
        resolved: Some(true),
        resolved_at: None,
        resolved_by: None,
        acked: None,
        acknowledged_at: None,
        owner_id: None,
        snoozed_until: None,
        escalation_level: None,
        severity: None,
    };
    store
        .update_incident(id, updates)
        .await
        .map_err(|e| e.to_string())?;
    Ok(json!({"status": "resolved", "id": id_str}))
}

pub async fn health(_store: &dyn Store, _workspace_id: Uuid, _params: Value) -> McpResult {
    Ok(json!({"status": "healthy", "version": "0.2.0"}))
}
