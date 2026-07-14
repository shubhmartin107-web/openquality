use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::Result;
use crate::types::*;

#[async_trait::async_trait]
pub trait Store: Send + Sync {
    async fn create_workspace(&self, name: &str, slug: &str) -> Result<WorkspaceRecord>;
    async fn get_workspace(&self, id: Uuid) -> Result<Option<WorkspaceRecord>>;
    async fn list_workspaces(&self) -> Result<Vec<WorkspaceRecord>>;

    async fn create_user(
        &self,
        workspace_id: Uuid,
        email: &str,
        name: &str,
        role: &str,
    ) -> Result<UserRecord>;
    async fn create_user_with_password(
        &self,
        workspace_id: Uuid,
        email: &str,
        name: &str,
        role: &str,
        password_hash: &str,
    ) -> Result<UserRecord>;
    async fn get_user(&self, id: Uuid) -> Result<Option<UserRecord>>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<UserRecord>>;
    async fn list_users(&self, workspace_id: Uuid) -> Result<Vec<UserRecord>>;

    async fn store_api_key(
        &self,
        user_id: Uuid,
        key_hash: &str,
        label: &str,
    ) -> Result<ApiKeyRecord>;
    async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKeyRecord>>;
    async fn list_api_keys(&self, user_id: Uuid) -> Result<Vec<ApiKeyRecord>>;
    async fn revoke_api_key(&self, id: Uuid) -> Result<()>;

    async fn create_data_source(
        &self,
        workspace_id: Uuid,
        name: &str,
        source_type: &str,
        config: serde_json::Value,
    ) -> Result<DataSourceRecord>;
    async fn get_data_source(&self, id: Uuid) -> Result<Option<DataSourceRecord>>;
    async fn list_data_sources(&self, workspace_id: Uuid) -> Result<Vec<DataSourceRecord>>;
    async fn delete_data_source(&self, id: Uuid) -> Result<()>;

    async fn create_suite(&self, workspace_id: Uuid, name: &str) -> Result<SuiteRecord>;
    async fn get_suite(&self, id: Uuid) -> Result<Option<SuiteRecord>>;
    async fn list_suites(&self, workspace_id: Uuid) -> Result<Vec<SuiteRecord>>;
    async fn delete_suite(&self, id: Uuid) -> Result<()>;

    async fn create_expectation(
        &self,
        suite_id: Uuid,
        expectation_type: &ExpectationType,
        column: Option<&str>,
    ) -> Result<ExpectationRecord>;
    async fn list_expectations(&self, suite_id: Uuid) -> Result<Vec<ExpectationRecord>>;

    async fn create_suite_run(&self, suite_id: Uuid, status: &str) -> Result<SuiteRunRecord>;
    async fn update_suite_run(
        &self,
        id: Uuid,
        status: &str,
        summary: &serde_json::Value,
    ) -> Result<()>;
    async fn list_suite_runs(&self, suite_id: Uuid) -> Result<Vec<SuiteRunRecord>>;

    async fn create_monitor(
        &self,
        workspace_id: Uuid,
        name: &str,
        monitor_type: &MonitorType,
        table_name: &str,
        config: serde_json::Value,
    ) -> Result<MonitorRecord>;
    async fn get_monitor(&self, id: Uuid) -> Result<Option<MonitorRecord>>;
    async fn list_monitors(&self, workspace_id: Uuid) -> Result<Vec<MonitorRecord>>;
    async fn list_monitors_all(&self) -> Result<Vec<MonitorRecord>>;
    async fn update_monitor(&self, id: Uuid, config: serde_json::Value) -> Result<()>;
    async fn delete_monitor(&self, id: Uuid) -> Result<()>;

    async fn append_monitor_history(
        &self,
        monitor_id: Uuid,
        score: f64,
        threshold: f64,
    ) -> Result<()>;
    async fn get_monitor_history(
        &self,
        monitor_id: Uuid,
        limit: i64,
    ) -> Result<Vec<MonitorHistoryRecord>>;

    async fn create_incident(&self, incident: &Incident) -> Result<()>;
    async fn get_incident(&self, id: Uuid) -> Result<Option<Incident>>;
    async fn list_incidents(
        &self,
        workspace_id: Uuid,
        status: Option<&str>,
    ) -> Result<Vec<Incident>>;
    async fn update_incident(&self, id: Uuid, updates: IncidentUpdate) -> Result<()>;

    async fn save_column_profile(&self, profile: &ColumnProfileRecord) -> Result<()>;
    async fn get_latest_profile(
        &self,
        data_source_id: Uuid,
        table_name: &str,
        column_name: &str,
    ) -> Result<Option<ColumnProfileRecord>>;
}

// ---------------------------------------------------------------------------
// Record types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRecord {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub email: String,
    pub name: String,
    pub role: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key_hash: String,
    pub label: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceRecord {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub source_type: String,
    pub config: serde_json::Value,
    pub connection_status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteRecord {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub data_source_id: Option<Uuid>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectationRecord {
    pub id: Uuid,
    pub suite_id: Uuid,
    pub expectation_type: ExpectationType,
    pub column: Option<String>,
    pub kwargs: serde_json::Value,
    pub tolerance: f64,
    pub position: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteRunRecord {
    pub id: Uuid,
    pub suite_id: Uuid,
    pub status: String,
    pub results_json: Option<serde_json::Value>,
    pub summary_json: Option<serde_json::Value>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorRecord {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub monitor_type: MonitorType,
    pub table_name: String,
    pub config: serde_json::Value,
    pub schedule_cron: Option<String>,
    pub enabled: bool,
    pub data_source_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorHistoryRecord {
    pub id: i64,
    pub monitor_id: Uuid,
    pub score: f64,
    pub threshold: f64,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnProfileRecord {
    pub data_source_id: Uuid,
    pub table_name: String,
    pub column_name: String,
    pub row_count: i64,
    pub null_count: i64,
    pub distinct_count: i64,
    pub min_val: Option<f64>,
    pub max_val: Option<f64>,
    pub mean_val: Option<f64>,
    pub stddev_val: Option<f64>,
    pub quantiles: Option<[f64; 5]>,
    pub profiled_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IncidentUpdate {
    pub resolved: Option<bool>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<Uuid>,
    pub acked: Option<bool>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub owner_id: Option<Uuid>,
    pub snoozed_until: Option<DateTime<Utc>>,
    pub escalation_level: Option<u32>,
    pub severity: Option<Severity>,
}

// ---------------------------------------------------------------------------
// InMemoryStore
// ---------------------------------------------------------------------------

pub struct InMemoryStore {
    workspaces: RwLock<HashMap<Uuid, WorkspaceRecord>>,
    users: RwLock<HashMap<Uuid, UserRecord>>,
    api_keys: RwLock<HashMap<Uuid, ApiKeyRecord>>,
    data_sources: RwLock<HashMap<Uuid, DataSourceRecord>>,
    suites: RwLock<HashMap<Uuid, SuiteRecord>>,
    expectations: RwLock<HashMap<Uuid, ExpectationRecord>>,
    suite_runs: RwLock<HashMap<Uuid, SuiteRunRecord>>,
    monitors: RwLock<HashMap<Uuid, MonitorRecord>>,
    monitor_history: RwLock<Vec<MonitorHistoryRecord>>,
    pub incidents: RwLock<Vec<Incident>>,
    column_profiles: RwLock<Vec<ColumnProfileRecord>>,
    next_history_id: RwLock<i64>,
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self {
            workspaces: RwLock::new(HashMap::new()),
            users: RwLock::new(HashMap::new()),
            api_keys: RwLock::new(HashMap::new()),
            data_sources: RwLock::new(HashMap::new()),
            suites: RwLock::new(HashMap::new()),
            expectations: RwLock::new(HashMap::new()),
            suite_runs: RwLock::new(HashMap::new()),
            monitors: RwLock::new(HashMap::new()),
            monitor_history: RwLock::new(Vec::new()),
            incidents: RwLock::new(Vec::new()),
            column_profiles: RwLock::new(Vec::new()),
            next_history_id: RwLock::new(1),
        }
    }
}

#[async_trait::async_trait]
impl Store for InMemoryStore {
    async fn create_workspace(&self, name: &str, slug: &str) -> Result<WorkspaceRecord> {
        let record = WorkspaceRecord {
            id: Uuid::new_v4(),
            name: name.to_string(),
            slug: slug.to_string(),
            created_at: Utc::now(),
        };
        self.workspaces.write().insert(record.id, record.clone());
        Ok(record)
    }

    async fn get_workspace(&self, id: Uuid) -> Result<Option<WorkspaceRecord>> {
        Ok(self.workspaces.read().get(&id).cloned())
    }

    async fn list_workspaces(&self) -> Result<Vec<WorkspaceRecord>> {
        Ok(self.workspaces.read().values().cloned().collect())
    }

    async fn create_user(
        &self,
        workspace_id: Uuid,
        email: &str,
        name: &str,
        role: &str,
    ) -> Result<UserRecord> {
        let record = UserRecord {
            id: Uuid::new_v4(),
            workspace_id,
            email: email.to_string(),
            name: name.to_string(),
            role: role.to_string(),
            password_hash: String::new(),
            created_at: Utc::now(),
        };
        self.users.write().insert(record.id, record.clone());
        Ok(record)
    }

    async fn create_user_with_password(
        &self,
        workspace_id: Uuid,
        email: &str,
        name: &str,
        role: &str,
        password_hash: &str,
    ) -> Result<UserRecord> {
        let record = UserRecord {
            id: Uuid::new_v4(),
            workspace_id,
            email: email.to_string(),
            name: name.to_string(),
            role: role.to_string(),
            password_hash: password_hash.to_string(),
            created_at: Utc::now(),
        };
        self.users.write().insert(record.id, record.clone());
        Ok(record)
    }

    async fn get_user(&self, id: Uuid) -> Result<Option<UserRecord>> {
        Ok(self.users.read().get(&id).cloned())
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<UserRecord>> {
        Ok(self
            .users
            .read()
            .values()
            .find(|u| u.email == email)
            .cloned())
    }

    async fn list_users(&self, workspace_id: Uuid) -> Result<Vec<UserRecord>> {
        Ok(self
            .users
            .read()
            .values()
            .filter(|u| u.workspace_id == workspace_id)
            .cloned()
            .collect())
    }

    async fn store_api_key(
        &self,
        user_id: Uuid,
        key_hash: &str,
        label: &str,
    ) -> Result<ApiKeyRecord> {
        let record = ApiKeyRecord {
            id: Uuid::new_v4(),
            user_id,
            key_hash: key_hash.to_string(),
            label: label.to_string(),
            created_at: Utc::now(),
            expires_at: None,
            revoked: false,
        };
        self.api_keys.write().insert(record.id, record.clone());
        Ok(record)
    }

    async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKeyRecord>> {
        Ok(self
            .api_keys
            .read()
            .values()
            .find(|k| k.key_hash == key_hash && !k.revoked)
            .cloned())
    }

    async fn revoke_api_key(&self, id: Uuid) -> Result<()> {
        if let Some(key) = self.api_keys.write().get_mut(&id) {
            key.revoked = true;
        }
        Ok(())
    }

    async fn list_api_keys(&self, user_id: Uuid) -> Result<Vec<ApiKeyRecord>> {
        Ok(self
            .api_keys
            .read()
            .values()
            .filter(|k| k.user_id == user_id && !k.revoked)
            .cloned()
            .collect())
    }

    async fn create_data_source(
        &self,
        workspace_id: Uuid,
        name: &str,
        source_type: &str,
        config: serde_json::Value,
    ) -> Result<DataSourceRecord> {
        let record = DataSourceRecord {
            id: Uuid::new_v4(),
            workspace_id,
            name: name.to_string(),
            source_type: source_type.to_string(),
            config,
            connection_status: "unknown".to_string(),
            created_at: Utc::now(),
        };
        self.data_sources.write().insert(record.id, record.clone());
        Ok(record)
    }

    async fn get_data_source(&self, id: Uuid) -> Result<Option<DataSourceRecord>> {
        Ok(self.data_sources.read().get(&id).cloned())
    }

    async fn list_data_sources(&self, workspace_id: Uuid) -> Result<Vec<DataSourceRecord>> {
        Ok(self
            .data_sources
            .read()
            .values()
            .filter(|d| d.workspace_id == workspace_id)
            .cloned()
            .collect())
    }

    async fn delete_data_source(&self, id: Uuid) -> Result<()> {
        self.data_sources.write().remove(&id);
        Ok(())
    }

    async fn create_suite(&self, workspace_id: Uuid, name: &str) -> Result<SuiteRecord> {
        let record = SuiteRecord {
            id: Uuid::new_v4(),
            workspace_id,
            name: name.to_string(),
            description: None,
            data_source_id: None,
            version: 1,
            created_at: Utc::now(),
        };
        self.suites.write().insert(record.id, record.clone());
        Ok(record)
    }

    async fn get_suite(&self, id: Uuid) -> Result<Option<SuiteRecord>> {
        Ok(self.suites.read().get(&id).cloned())
    }

    async fn list_suites(&self, workspace_id: Uuid) -> Result<Vec<SuiteRecord>> {
        Ok(self
            .suites
            .read()
            .values()
            .filter(|s| s.workspace_id == workspace_id)
            .cloned()
            .collect())
    }

    async fn delete_suite(&self, id: Uuid) -> Result<()> {
        self.suites.write().remove(&id);
        Ok(())
    }

    async fn create_expectation(
        &self,
        suite_id: Uuid,
        expectation_type: &ExpectationType,
        column: Option<&str>,
    ) -> Result<ExpectationRecord> {
        let position = {
            let exs = self.expectations.read();
            exs.values()
                .filter(|e: &&ExpectationRecord| e.suite_id == suite_id)
                .count() as i32
        };
        let record = ExpectationRecord {
            id: Uuid::new_v4(),
            suite_id,
            expectation_type: expectation_type.clone(),
            column: column.map(String::from),
            kwargs: serde_json::Value::Object(serde_json::Map::new()),
            tolerance: 0.0,
            position,
        };
        self.expectations.write().insert(record.id, record.clone());
        Ok(record)
    }

    async fn list_expectations(&self, suite_id: Uuid) -> Result<Vec<ExpectationRecord>> {
        Ok(self
            .expectations
            .read()
            .values()
            .filter(|e| e.suite_id == suite_id)
            .cloned()
            .collect())
    }

    async fn create_suite_run(&self, suite_id: Uuid, status: &str) -> Result<SuiteRunRecord> {
        let record = SuiteRunRecord {
            id: Uuid::new_v4(),
            suite_id,
            status: status.to_string(),
            results_json: None,
            summary_json: None,
            started_at: Utc::now(),
            finished_at: None,
        };
        self.suite_runs.write().insert(record.id, record.clone());
        Ok(record)
    }

    async fn update_suite_run(
        &self,
        id: Uuid,
        status: &str,
        summary: &serde_json::Value,
    ) -> Result<()> {
        if let Some(run) = self.suite_runs.write().get_mut(&id) {
            run.status = status.to_string();
            run.summary_json = Some(summary.clone());
            run.finished_at = Some(Utc::now());
        }
        Ok(())
    }

    async fn list_suite_runs(&self, suite_id: Uuid) -> Result<Vec<SuiteRunRecord>> {
        Ok(self
            .suite_runs
            .read()
            .values()
            .filter(|r| r.suite_id == suite_id)
            .cloned()
            .collect())
    }

    async fn create_monitor(
        &self,
        workspace_id: Uuid,
        name: &str,
        monitor_type: &MonitorType,
        table_name: &str,
        config: serde_json::Value,
    ) -> Result<MonitorRecord> {
        let record = MonitorRecord {
            id: Uuid::new_v4(),
            workspace_id,
            name: name.to_string(),
            monitor_type: monitor_type.clone(),
            table_name: table_name.to_string(),
            config,
            schedule_cron: None,
            enabled: true,
            data_source_id: None,
            created_at: Utc::now(),
        };
        self.monitors.write().insert(record.id, record.clone());
        Ok(record)
    }

    async fn get_monitor(&self, id: Uuid) -> Result<Option<MonitorRecord>> {
        Ok(self.monitors.read().get(&id).cloned())
    }

    async fn list_monitors(&self, workspace_id: Uuid) -> Result<Vec<MonitorRecord>> {
        Ok(self
            .monitors
            .read()
            .values()
            .filter(|m| m.workspace_id == workspace_id)
            .cloned()
            .collect())
    }

    async fn list_monitors_all(&self) -> Result<Vec<MonitorRecord>> {
        Ok(self.monitors.read().values().cloned().collect())
    }

    async fn update_monitor(&self, id: Uuid, config: serde_json::Value) -> Result<()> {
        if let Some(mon) = self.monitors.write().get_mut(&id) {
            if let Some(enabled) = config.get("enabled").and_then(|v| v.as_bool()) {
                mon.enabled = enabled;
            }
            if let Some(cron) = config.get("schedule_cron").and_then(|v| v.as_str()) {
                mon.schedule_cron = Some(cron.to_string());
            }
        }
        Ok(())
    }

    async fn delete_monitor(&self, id: Uuid) -> Result<()> {
        self.monitors.write().remove(&id);
        Ok(())
    }

    async fn append_monitor_history(
        &self,
        monitor_id: Uuid,
        score: f64,
        threshold: f64,
    ) -> Result<()> {
        let id = {
            let mut ctr = self.next_history_id.write();
            let id = *ctr;
            *ctr += 1;
            id
        };
        let record = MonitorHistoryRecord {
            id,
            monitor_id,
            score,
            threshold,
            checked_at: Utc::now(),
        };
        self.monitor_history.write().push(record);
        Ok(())
    }

    async fn get_monitor_history(
        &self,
        monitor_id: Uuid,
        limit: i64,
    ) -> Result<Vec<MonitorHistoryRecord>> {
        let hist = self.monitor_history.read();
        let mut results: Vec<_> = hist
            .iter()
            .filter(|h| h.monitor_id == monitor_id)
            .cloned()
            .collect();
        results.reverse();
        results.truncate(limit as usize);
        Ok(results)
    }

    async fn create_incident(&self, incident: &Incident) -> Result<()> {
        self.incidents.write().push(incident.clone());
        Ok(())
    }

    async fn get_incident(&self, id: Uuid) -> Result<Option<Incident>> {
        Ok(self.incidents.read().iter().find(|i| i.id == id).cloned())
    }

    async fn list_incidents(
        &self,
        _workspace_id: Uuid,
        status: Option<&str>,
    ) -> Result<Vec<Incident>> {
        let guard = self.incidents.read();
        Ok(match status {
            Some("open") => guard.iter().filter(|i| !i.resolved).cloned().collect(),
            Some("resolved") => guard.iter().filter(|i| i.resolved).cloned().collect(),
            _ => guard.clone(),
        })
    }

    async fn update_incident(&self, id: Uuid, updates: IncidentUpdate) -> Result<()> {
        let mut guard = self.incidents.write();
        if let Some(inc) = guard.iter_mut().find(|i| i.id == id) {
            if let Some(v) = updates.resolved {
                inc.resolved = v;
            }
            if let Some(v) = updates.resolved_at {
                inc.resolved_at = Some(v);
            }
            if let Some(v) = updates.resolved_by {
                inc.owner_id = Some(v);
            }
            if let Some(v) = updates.acked {
                inc.acked = v;
            }
            if let Some(v) = updates.acknowledged_at {
                inc.acknowledged_at = Some(v);
            }
            if let Some(v) = updates.owner_id {
                inc.owner_id = Some(v);
            }
            if let Some(v) = updates.snoozed_until {
                inc.snoozed_until = Some(v);
            }
            if let Some(v) = updates.escalation_level {
                inc.escalation_level = v;
            }
            if let Some(v) = updates.severity {
                inc.severity = v;
            }
        }
        Ok(())
    }

    async fn save_column_profile(&self, profile: &ColumnProfileRecord) -> Result<()> {
        self.column_profiles.write().push(profile.clone());
        Ok(())
    }

    async fn get_latest_profile(
        &self,
        data_source_id: Uuid,
        table_name: &str,
        column_name: &str,
    ) -> Result<Option<ColumnProfileRecord>> {
        let guard = self.column_profiles.read();
        Ok(guard
            .iter()
            .rfind(|p| {
                p.data_source_id == data_source_id
                    && p.table_name == table_name
                    && p.column_name == column_name
            })
            .cloned())
    }
}
