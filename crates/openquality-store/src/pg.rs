use chrono::{DateTime, Utc};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

use openquality_core::error::Result;
use openquality_core::store::*;
use openquality_core::types::*;

pub struct PgStore {
    pool: PgPool,
}

impl PgStore {
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .connect(database_url)
            .await
            .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        let sql = include_str!("migrations/001_init.sql");
        sqlx::query(sql)
            .execute(&self.pool)
            .await
            .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))?;
        Ok(())
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

impl PgStore {
    fn serialize_monitor_type(mt: &MonitorType) -> String {
        serde_json::to_string(mt).unwrap_or_default()
    }

    fn deserialize_monitor_type(s: &str) -> Option<MonitorType> {
        serde_json::from_str(s).ok()
    }
}

#[async_trait::async_trait]
impl Store for PgStore {
    async fn create_workspace(&self, name: &str, slug: &str) -> Result<WorkspaceRecord> {
        let id = Uuid::new_v4();
        sqlx::query_as::<_, (Uuid, String, String, DateTime<Utc>)>(
            "INSERT INTO workspaces (id, name, slug) VALUES ($1, $2, $3) RETURNING id, name, slug, created_at",
        )
        .bind(id)
        .bind(name)
        .bind(slug)
        .fetch_one(&self.pool)
        .await
        .map(|(id, name, slug, created_at)| WorkspaceRecord { id, name, slug, created_at })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn get_workspace(&self, id: Uuid) -> Result<Option<WorkspaceRecord>> {
        sqlx::query_as::<_, (Uuid, String, String, DateTime<Utc>)>(
            "SELECT id, name, slug, created_at FROM workspaces WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map(|opt| {
            opt.map(|(id, name, slug, created_at)| WorkspaceRecord {
                id,
                name,
                slug,
                created_at,
            })
        })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn list_workspaces(&self) -> Result<Vec<WorkspaceRecord>> {
        sqlx::query_as::<_, (Uuid, String, String, DateTime<Utc>)>(
            "SELECT id, name, slug, created_at FROM workspaces ORDER BY created_at",
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|(id, name, slug, created_at)| WorkspaceRecord {
                    id,
                    name,
                    slug,
                    created_at,
                })
                .collect()
        })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn create_user(
        &self,
        workspace_id: Uuid,
        email: &str,
        name: &str,
        role: &str,
    ) -> Result<UserRecord> {
        let id = Uuid::new_v4();
        sqlx::query_as::<_, (Uuid, Uuid, String, String, String, String, DateTime<Utc>)>(
            "INSERT INTO users (id, workspace_id, email, name, role, password_hash) VALUES ($1, $2, $3, $4, $5, '') RETURNING id, workspace_id, email, name, role, password_hash, created_at",
        )
        .bind(id)
        .bind(workspace_id)
        .bind(email)
        .bind(name)
        .bind(role)
        .fetch_one(&self.pool)
        .await
        .map(|(id, workspace_id, email, name, role, password_hash, created_at)| UserRecord { id, workspace_id, email, name, role, password_hash, created_at })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn create_user_with_password(
        &self,
        workspace_id: Uuid,
        email: &str,
        name: &str,
        role: &str,
        password_hash: &str,
    ) -> Result<UserRecord> {
        let id = Uuid::new_v4();
        sqlx::query_as::<_, (Uuid, Uuid, String, String, String, String, DateTime<Utc>)>(
            "INSERT INTO users (id, workspace_id, email, name, role, password_hash) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, workspace_id, email, name, role, password_hash, created_at",
        )
        .bind(id)
        .bind(workspace_id)
        .bind(email)
        .bind(name)
        .bind(role)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await
        .map(|(id, workspace_id, email, name, role, password_hash, created_at)| UserRecord { id, workspace_id, email, name, role, password_hash, created_at })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn get_user(&self, id: Uuid) -> Result<Option<UserRecord>> {
        sqlx::query_as::<_, (Uuid, Uuid, String, String, String, String, DateTime<Utc>)>(
            "SELECT id, workspace_id, email, name, role, password_hash, created_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map(|opt| {
            opt.map(
                |(id, workspace_id, email, name, role, password_hash, created_at)| UserRecord {
                    id,
                    workspace_id,
                    email,
                    name,
                    role,
                    password_hash,
                    created_at,
                },
            )
        })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<UserRecord>> {
        sqlx::query_as::<_, (Uuid, Uuid, String, String, String, String, DateTime<Utc>)>(
            "SELECT id, workspace_id, email, name, role, password_hash, created_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map(|opt| {
            opt.map(
                |(id, workspace_id, email, name, role, password_hash, created_at)| UserRecord {
                    id,
                    workspace_id,
                    email,
                    name,
                    role,
                    password_hash,
                    created_at,
                },
            )
        })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn list_users(&self, workspace_id: Uuid) -> Result<Vec<UserRecord>> {
        sqlx::query_as::<_, (Uuid, Uuid, String, String, String, String, DateTime<Utc>)>(
            "SELECT id, workspace_id, email, name, role, password_hash, created_at FROM users WHERE workspace_id = $1 ORDER BY created_at",
        )
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|(id, workspace_id, email, name, role, password_hash, created_at)| UserRecord { id, workspace_id, email, name, role, password_hash, created_at })
                .collect()
        })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn store_api_key(
        &self,
        user_id: Uuid,
        key_hash: &str,
        label: &str,
    ) -> Result<ApiKeyRecord> {
        let id = Uuid::new_v4();
        sqlx::query_as::<_, (Uuid, Uuid, String, String, DateTime<Utc>, Option<DateTime<Utc>>, bool)>(
            "INSERT INTO api_keys (id, user_id, key_hash, label) VALUES ($1, $2, $3, $4) RETURNING id, user_id, key_hash, label, created_at, expires_at, revoked",
        )
        .bind(id)
        .bind(user_id)
        .bind(key_hash)
        .bind(label)
        .fetch_one(&self.pool)
        .await
        .map(|(id, user_id, key_hash, label, created_at, expires_at, revoked)| ApiKeyRecord { id, user_id, key_hash, label, created_at, expires_at, revoked })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKeyRecord>> {
        sqlx::query_as::<_, (Uuid, Uuid, String, String, DateTime<Utc>, Option<DateTime<Utc>>, bool)>(
            "SELECT id, user_id, key_hash, label, created_at, expires_at, revoked FROM api_keys WHERE key_hash = $1 AND revoked = FALSE",
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await
        .map(|opt| opt.map(|(id, user_id, key_hash, label, created_at, expires_at, revoked)| ApiKeyRecord { id, user_id, key_hash, label, created_at, expires_at, revoked }))
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn list_api_keys(&self, user_id: Uuid) -> Result<Vec<ApiKeyRecord>> {
        sqlx::query_as::<_, (Uuid, Uuid, String, String, DateTime<Utc>, Option<DateTime<Utc>>, bool)>(
            "SELECT id, user_id, key_hash, label, created_at, expires_at, revoked FROM api_keys WHERE user_id = $1 AND revoked = FALSE ORDER BY created_at",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|(id, user_id, key_hash, label, created_at, expires_at, revoked)| ApiKeyRecord { id, user_id, key_hash, label, created_at, expires_at, revoked })
                .collect()
        })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn revoke_api_key(&self, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE api_keys SET revoked = TRUE WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))?;
        Ok(())
    }

    async fn create_data_source(
        &self,
        workspace_id: Uuid,
        name: &str,
        source_type: &str,
        config: serde_json::Value,
    ) -> Result<DataSourceRecord> {
        let id = Uuid::new_v4();
        sqlx::query_as::<_, (Uuid, Uuid, String, String, serde_json::Value, String, DateTime<Utc>)>(
            "INSERT INTO data_sources (id, workspace_id, name, source_type, config_json) VALUES ($1, $2, $3, $4, $5) RETURNING id, workspace_id, name, source_type, config_json, connection_status, created_at",
        )
        .bind(id)
        .bind(workspace_id)
        .bind(name)
        .bind(source_type)
        .bind(&config)
        .fetch_one(&self.pool)
        .await
        .map(|(id, workspace_id, name, source_type, config, connection_status, created_at)| DataSourceRecord { id, workspace_id, name, source_type, config, connection_status, created_at })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn get_data_source(&self, id: Uuid) -> Result<Option<DataSourceRecord>> {
        sqlx::query_as::<_, (Uuid, Uuid, String, String, serde_json::Value, String, DateTime<Utc>)>(
            "SELECT id, workspace_id, name, source_type, config_json, connection_status, created_at FROM data_sources WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map(|opt| opt.map(|(id, workspace_id, name, source_type, config, connection_status, created_at)| DataSourceRecord { id, workspace_id, name, source_type, config, connection_status, created_at }))
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn list_data_sources(&self, workspace_id: Uuid) -> Result<Vec<DataSourceRecord>> {
        sqlx::query_as::<_, (Uuid, Uuid, String, String, serde_json::Value, String, DateTime<Utc>)>(
            "SELECT id, workspace_id, name, source_type, config_json, connection_status, created_at FROM data_sources WHERE workspace_id = $1 ORDER BY created_at",
        )
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|(id, workspace_id, name, source_type, config, connection_status, created_at)| DataSourceRecord { id, workspace_id, name, source_type, config, connection_status, created_at })
                .collect()
        })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn delete_data_source(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM data_sources WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))?;
        Ok(())
    }

    async fn create_suite(&self, workspace_id: Uuid, name: &str) -> Result<SuiteRecord> {
        let id = Uuid::new_v4();
        sqlx::query_as::<_, (Uuid, Uuid, String, Option<String>, Option<Uuid>, i32, DateTime<Utc>)>(
            "INSERT INTO expectation_suites (id, workspace_id, name) VALUES ($1, $2, $3) RETURNING id, workspace_id, name, description, data_source_id, version, created_at",
        )
        .bind(id)
        .bind(workspace_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map(|(id, workspace_id, name, description, data_source_id, version, created_at)| SuiteRecord { id, workspace_id, name, description, data_source_id, version, created_at })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn get_suite(&self, id: Uuid) -> Result<Option<SuiteRecord>> {
        sqlx::query_as::<_, (Uuid, Uuid, String, Option<String>, Option<Uuid>, i32, DateTime<Utc>)>(
            "SELECT id, workspace_id, name, description, data_source_id, version, created_at FROM expectation_suites WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map(|opt| opt.map(|(id, workspace_id, name, description, data_source_id, version, created_at)| SuiteRecord { id, workspace_id, name, description, data_source_id, version, created_at }))
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn list_suites(&self, workspace_id: Uuid) -> Result<Vec<SuiteRecord>> {
        sqlx::query_as::<_, (Uuid, Uuid, String, Option<String>, Option<Uuid>, i32, DateTime<Utc>)>(
            "SELECT id, workspace_id, name, description, data_source_id, version, created_at FROM expectation_suites WHERE workspace_id = $1 ORDER BY created_at",
        )
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|(id, workspace_id, name, description, data_source_id, version, created_at)| SuiteRecord { id, workspace_id, name, description, data_source_id, version, created_at })
                .collect()
        })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn delete_suite(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM expectation_suites WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))?;
        Ok(())
    }

    async fn create_expectation(
        &self,
        suite_id: Uuid,
        expectation_type: &ExpectationType,
        column: Option<&str>,
    ) -> Result<ExpectationRecord> {
        let id = Uuid::new_v4();
        let type_str = format!("{:?}", expectation_type);
        let position: i32 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM expectations WHERE suite_id = $1",
        )
        .bind(suite_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))?;
        sqlx::query_as::<_, (Uuid, Uuid, String, Option<String>, serde_json::Value, f64, i32)>(
            "INSERT INTO expectations (id, suite_id, expectation_type, column_name, kwargs_json, tolerance, position) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id, suite_id, expectation_type, column_name, kwargs_json, tolerance, position",
        )
        .bind(id)
        .bind(suite_id)
        .bind(&type_str)
        .bind(column)
        .bind(position)
        .fetch_one(&self.pool)
        .await
        .map(|(id, suite_id, _type_str, column_name, kwargs, tolerance, position)| {
            ExpectationRecord {
                id,
                suite_id,
                expectation_type: expectation_type.clone(),
                column: column_name,
                kwargs,
                tolerance,
                position,
            }
        })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn list_expectations(&self, suite_id: Uuid) -> Result<Vec<ExpectationRecord>> {
        sqlx::query_as::<_, (Uuid, Uuid, String, Option<String>, serde_json::Value, f64, i32)>(
            "SELECT id, suite_id, expectation_type, column_name, kwargs_json, tolerance, position FROM expectations WHERE suite_id = $1 ORDER BY position",
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|(id, suite_id, type_str, column_name, kwargs, tolerance, position)| {
                    let expectation_type = parse_expectation_type_str(&type_str).unwrap_or(ExpectationType::NotNull);
                    ExpectationRecord {
                        id, suite_id,
                        expectation_type,
                        column: column_name, kwargs, tolerance, position,
                    }
                })
                .collect()
        })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn create_suite_run(&self, suite_id: Uuid, status: &str) -> Result<SuiteRunRecord> {
        let id = Uuid::new_v4();
        sqlx::query_as::<_, (Uuid, Uuid, String, Option<serde_json::Value>, Option<serde_json::Value>, DateTime<Utc>, Option<DateTime<Utc>>)>(
            "INSERT INTO suite_runs (id, suite_id, status) VALUES ($1, $2, $3) RETURNING id, suite_id, status, results_json, summary_json, started_at, finished_at",
        )
        .bind(id)
        .bind(suite_id)
        .bind(status)
        .fetch_one(&self.pool)
        .await
        .map(|(id, suite_id, status, results_json, summary_json, started_at, finished_at)| SuiteRunRecord { id, suite_id, status, results_json, summary_json, started_at, finished_at })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn update_suite_run(
        &self,
        id: Uuid,
        status: &str,
        summary: &serde_json::Value,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE suite_runs SET status = $1, summary_json = $2, finished_at = NOW() WHERE id = $3",
        )
        .bind(status)
        .bind(summary)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))?;
        Ok(())
    }

    async fn list_suite_runs(&self, suite_id: Uuid) -> Result<Vec<SuiteRunRecord>> {
        sqlx::query_as::<_, (Uuid, Uuid, String, Option<serde_json::Value>, Option<serde_json::Value>, DateTime<Utc>, Option<DateTime<Utc>>)>(
            "SELECT id, suite_id, status, results_json, summary_json, started_at, finished_at FROM suite_runs WHERE suite_id = $1 ORDER BY started_at DESC",
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|(id, suite_id, status, results_json, summary_json, started_at, finished_at)| SuiteRunRecord { id, suite_id, status, results_json, summary_json, started_at, finished_at })
                .collect()
        })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn create_monitor(
        &self,
        workspace_id: Uuid,
        name: &str,
        monitor_type: &MonitorType,
        table_name: &str,
        config: serde_json::Value,
    ) -> Result<MonitorRecord> {
        let id = Uuid::new_v4();
        let type_json = Self::serialize_monitor_type(monitor_type);
        sqlx::query_as::<_, (Uuid, Uuid, String, String, String, serde_json::Value, Option<String>, bool, Option<Uuid>, DateTime<Utc>)>(
            "INSERT INTO monitors (id, workspace_id, name, monitor_type, table_name, config_json) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, workspace_id, name, monitor_type, table_name, config_json, schedule_cron, enabled, data_source_id, created_at",
        )
        .bind(id)
        .bind(workspace_id)
        .bind(name)
        .bind(&type_json)
        .bind(table_name)
        .bind(&config)
        .fetch_one(&self.pool)
        .await
        .map(|(id, workspace_id, name, type_str, table_name, config, schedule_cron, enabled, data_source_id, created_at)| {
            let mt = Self::deserialize_monitor_type(&type_str).unwrap_or(MonitorType::Schema);
            MonitorRecord {
                id, workspace_id, name,
                monitor_type: mt,
                table_name, config, schedule_cron, enabled, data_source_id, created_at,
            }
        })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn get_monitor(&self, id: Uuid) -> Result<Option<MonitorRecord>> {
        sqlx::query_as::<_, (Uuid, Uuid, String, String, String, serde_json::Value, Option<String>, bool, Option<Uuid>, DateTime<Utc>)>(
            "SELECT id, workspace_id, name, monitor_type, table_name, config_json, schedule_cron, enabled, data_source_id, created_at FROM monitors WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map(|opt| {
            opt.map(|(id, workspace_id, name, type_str, table_name, config, schedule_cron, enabled, data_source_id, created_at)| {
                let mt = Self::deserialize_monitor_type(&type_str).unwrap_or(MonitorType::Schema);
                MonitorRecord {
                    id, workspace_id, name,
                    monitor_type: mt,
                    table_name, config, schedule_cron, enabled, data_source_id, created_at,
                }
            })
        })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn list_monitors_all(&self) -> Result<Vec<MonitorRecord>> {
        sqlx::query_as::<_, (Uuid, Uuid, String, String, String, serde_json::Value, Option<String>, bool, Option<Uuid>, DateTime<Utc>)>(
            "SELECT id, workspace_id, name, monitor_type, table_name, config_json, schedule_cron, enabled, data_source_id, created_at FROM monitors ORDER BY created_at",
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|(id, workspace_id, name, type_str, table_name, config, schedule_cron, enabled, data_source_id, created_at)| {
                    let mt = Self::deserialize_monitor_type(&type_str).unwrap_or(MonitorType::Schema);
                    MonitorRecord {
                        id, workspace_id, name,
                        monitor_type: mt,
                        table_name, config, schedule_cron, enabled, data_source_id, created_at,
                    }
                })
                .collect()
        })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn list_monitors(&self, workspace_id: Uuid) -> Result<Vec<MonitorRecord>> {
        sqlx::query_as::<_, (Uuid, Uuid, String, String, String, serde_json::Value, Option<String>, bool, Option<Uuid>, DateTime<Utc>)>(
            "SELECT id, workspace_id, name, monitor_type, table_name, config_json, schedule_cron, enabled, data_source_id, created_at FROM monitors WHERE workspace_id = $1 ORDER BY created_at",
        )
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|(id, workspace_id, name, type_str, table_name, config, schedule_cron, enabled, data_source_id, created_at)| {
                    let mt = Self::deserialize_monitor_type(&type_str).unwrap_or(MonitorType::Schema);
                    MonitorRecord {
                        id, workspace_id, name,
                        monitor_type: mt,
                        table_name, config, schedule_cron, enabled, data_source_id, created_at,
                    }
                })
                .collect()
        })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn update_monitor(&self, id: Uuid, config: serde_json::Value) -> Result<()> {
        let enabled = config.get("enabled").and_then(|v| v.as_bool());
        let cron = config.get("schedule_cron").and_then(|v| v.as_str());
        if enabled.is_none() && cron.is_none() {
            return Ok(());
        }
        sqlx::query(
            "UPDATE monitors SET enabled = COALESCE($1, enabled), schedule_cron = COALESCE($2, schedule_cron) WHERE id = $3",
        )
        .bind(enabled)
        .bind(cron)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))?;
        Ok(())
    }

    async fn delete_monitor(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM monitors WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))?;
        Ok(())
    }

    async fn append_monitor_history(
        &self,
        monitor_id: Uuid,
        score: f64,
        threshold: f64,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO monitor_history (monitor_id, score, threshold) VALUES ($1, $2, $3)",
        )
        .bind(monitor_id)
        .bind(score)
        .bind(threshold)
        .execute(&self.pool)
        .await
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))?;
        Ok(())
    }

    async fn get_monitor_history(
        &self,
        monitor_id: Uuid,
        limit: i64,
    ) -> Result<Vec<MonitorHistoryRecord>> {
        sqlx::query_as::<_, (i64, Uuid, f64, f64, DateTime<Utc>)>(
            "SELECT id, monitor_id, score, threshold, checked_at FROM monitor_history WHERE monitor_id = $1 ORDER BY checked_at DESC LIMIT $2",
        )
        .bind(monitor_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|(id, monitor_id, score, threshold, checked_at)| MonitorHistoryRecord { id, monitor_id, score, threshold, checked_at })
                .collect()
        })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn create_incident(&self, incident: &Incident) -> Result<()> {
        let hints = serde_json::to_value(&incident.root_cause_hints).unwrap_or_default();
        let detail = serde_json::to_value(&incident.monitor_result).unwrap_or_default();
        let status = if incident.resolved {
            "resolved"
        } else {
            "open"
        };
        sqlx::query(
            "INSERT INTO incidents (id, workspace_id, monitor_id, severity, status, message, group_key, root_cause_hints_json, detail_json, escalation_level, snoozed_until, created_at, resolved_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
        )
        .bind(incident.id)
        .bind(incident.workspace_id)
        .bind(&incident.monitor_id)
        .bind(format!("{}", incident.severity).to_lowercase())
        .bind(status)
        .bind(&incident.message)
        .bind(&incident.group_key)
        .bind(&hints)
        .bind(&detail)
        .bind(incident.escalation_level as i32)
        .bind(incident.snoozed_until)
        .bind(incident.timestamp)
        .bind(incident.resolved_at)
        .execute(&self.pool)
        .await
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))?;
        Ok(())
    }

    async fn get_incident(&self, id: Uuid) -> Result<Option<Incident>> {
        sqlx::query_as::<_, (Uuid, Option<Uuid>, String, String, String, String, Option<String>, Option<serde_json::Value>, Option<serde_json::Value>, i32, Option<DateTime<Utc>>, DateTime<Utc>, Option<DateTime<Utc>>)>(
            "SELECT id, workspace_id, monitor_id, severity, status, message, group_key, root_cause_hints_json, detail_json, escalation_level, snoozed_until, created_at, resolved_at FROM incidents WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map(|opt| {
            opt.map(|(_id, _ws, _mid, _sev, _status, _msg, _gk, _hints, _detail, _esc, _snoozed, _ts, _resolved_at)| {
                let mid = _mid.clone();
                let stored_result: MonitorResult = _detail
                    .as_ref()
                    .and_then(|d| serde_json::from_value(d.clone()).ok())
                    .unwrap_or(MonitorResult {
                        monitor_id: mid.clone(),
                        monitor_type: MonitorType::Schema,
                        table_name: String::new(),
                        alert: false,
                        severity: Severity::Warning,
                        score: 0.0,
                        threshold: 0.0,
                        message: _msg.clone(),
                        details: std::collections::HashMap::new(),
                        timestamp: _ts,
                    });
                Incident {
                    id: _id,
                    workspace_id: _ws,
                    monitor_id: mid.clone(),
                    monitor_result: stored_result,
                    severity: if _sev == "critical" { Severity::Critical } else if _sev == "info" { Severity::Info } else { Severity::Warning },
                    message: _msg,
                    detail: _detail.unwrap_or_default(),
                    root_cause_hints: _hints.and_then(|h| serde_json::from_value(h).ok()).unwrap_or_default(),
                    timestamp: _ts,
                    resolved: _status == "resolved",
                    resolved_at: _resolved_at,
                    acked: false,
                    acknowledged_at: None,
                    group_key: _gk,
                    escalation_level: _esc as u32,
                    owner_id: None,
                    snoozed_until: _snoozed,
                }
            })
        })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }

    async fn list_incidents(
        &self,
        workspace_id: Uuid,
        status: Option<&str>,
    ) -> Result<Vec<Incident>> {
        let rows = if let Some(s) = status {
            sqlx::query_as::<_, (Uuid, Option<Uuid>, String, String, String, String, Option<String>, Option<serde_json::Value>, Option<serde_json::Value>, i32, Option<DateTime<Utc>>, DateTime<Utc>, Option<DateTime<Utc>>)>(
                "SELECT id, workspace_id, monitor_id, severity, status, message, group_key, root_cause_hints_json, detail_json, escalation_level, snoozed_until, created_at, resolved_at FROM incidents WHERE workspace_id = $1 AND status = $2 ORDER BY created_at DESC",
            )
            .bind(workspace_id)
            .bind(s)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))?
        } else {
            sqlx::query_as::<_, (Uuid, Option<Uuid>, String, String, String, String, Option<String>, Option<serde_json::Value>, Option<serde_json::Value>, i32, Option<DateTime<Utc>>, DateTime<Utc>, Option<DateTime<Utc>>)>(
                "SELECT id, workspace_id, monitor_id, severity, status, message, group_key, root_cause_hints_json, detail_json, escalation_level, snoozed_until, created_at, resolved_at FROM incidents WHERE workspace_id = $1 ORDER BY created_at DESC",
            )
            .bind(workspace_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))?
        };
        Ok(rows
            .into_iter()
            .map(
                |(
                    _id,
                    _ws,
                    _mid,
                    _sev,
                    _status,
                    _msg,
                    _gk,
                    _hints,
                    _detail,
                    _esc,
                    _snoozed,
                    _ts,
                    _resolved_at,
                )| {
                    let mid = _mid.clone();
                    let stored_result: MonitorResult = _detail
                        .as_ref()
                        .and_then(|d| serde_json::from_value(d.clone()).ok())
                        .unwrap_or(MonitorResult {
                            monitor_id: mid.clone(),
                            monitor_type: MonitorType::Schema,
                            table_name: String::new(),
                            alert: false,
                            severity: Severity::Warning,
                            score: 0.0,
                            threshold: 0.0,
                            message: _msg.clone(),
                            details: std::collections::HashMap::new(),
                            timestamp: _ts,
                        });
                    Incident {
                        id: _id,
                        workspace_id: _ws,
                        monitor_id: mid.clone(),
                        monitor_result: stored_result,
                        severity: if _sev == "critical" {
                            Severity::Critical
                        } else if _sev == "info" {
                            Severity::Info
                        } else {
                            Severity::Warning
                        },
                        message: _msg,
                        detail: _detail.unwrap_or_default(),
                        root_cause_hints: _hints
                            .and_then(|h| serde_json::from_value(h).ok())
                            .unwrap_or_default(),
                        timestamp: _ts,
                        resolved: _status == "resolved",
                        resolved_at: _resolved_at,
                        acked: false,
                        acknowledged_at: None,
                        group_key: _gk,
                        escalation_level: _esc as u32,
                        owner_id: None,
                        snoozed_until: _snoozed,
                    }
                },
            )
            .collect())
    }

    async fn update_incident(&self, id: Uuid, updates: IncidentUpdate) -> Result<()> {
        if updates.resolved.unwrap_or(false) {
            sqlx::query("UPDATE incidents SET status = 'resolved', resolved_at = $1 WHERE id = $2")
                .bind(updates.resolved_at)
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))?;
        }
        if updates.acked.unwrap_or(false) {
            sqlx::query("UPDATE incidents SET owner_id = $1 WHERE id = $2")
                .bind(updates.owner_id)
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))?;
        }
        if let Some(snoozed) = updates.snoozed_until {
            sqlx::query("UPDATE incidents SET snoozed_until = $1 WHERE id = $2")
                .bind(snoozed)
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))?;
        }
        if let Some(esc) = updates.escalation_level {
            sqlx::query("UPDATE incidents SET escalation_level = $1 WHERE id = $2")
                .bind(esc as i32)
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))?;
        }
        Ok(())
    }

    async fn save_column_profile(&self, profile: &ColumnProfileRecord) -> Result<()> {
        sqlx::query(
            "INSERT INTO column_profiles (data_source_id, table_name, column_name, row_count, null_count, distinct_count, min_val, max_val, mean_val, stddev_val, profiled_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        )
        .bind(profile.data_source_id)
        .bind(&profile.table_name)
        .bind(&profile.column_name)
        .bind(profile.row_count)
        .bind(profile.null_count)
        .bind(profile.distinct_count)
        .bind(profile.min_val)
        .bind(profile.max_val)
        .bind(profile.mean_val)
        .bind(profile.stddev_val)
        .bind(profile.profiled_at)
        .execute(&self.pool)
        .await
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))?;
        Ok(())
    }

    async fn get_latest_profile(
        &self,
        data_source_id: Uuid,
        table_name: &str,
        column_name: &str,
    ) -> Result<Option<ColumnProfileRecord>> {
        sqlx::query_as::<_, (Uuid, String, String, i64, i64, i64, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<serde_json::Value>, DateTime<Utc>)>(
            "SELECT data_source_id, table_name, column_name, row_count, null_count, distinct_count, min_val, max_val, mean_val, stddev_val, quantiles_json, profiled_at FROM column_profiles WHERE data_source_id = $1 AND table_name = $2 AND column_name = $3 ORDER BY profiled_at DESC LIMIT 1",
        )
        .bind(data_source_id)
        .bind(table_name)
        .bind(column_name)
        .fetch_optional(&self.pool)
        .await
        .map(|opt| {
            opt.map(|(ds_id, tbl, col, rc, nc, dc, min, max, mean, stddev, _quantiles_json, profiled_at)| {
                ColumnProfileRecord {
                    data_source_id: ds_id,
                    table_name: tbl,
                    column_name: col,
                    row_count: rc,
                    null_count: nc,
                    distinct_count: dc,
                    min_val: min,
                    max_val: max,
                    mean_val: mean,
                    stddev_val: stddev,
                    quantiles: None,
                    profiled_at,
                }
            })
        })
        .map_err(|e| openquality_core::error::OpenQualityError::Store(e.to_string()))
    }
}

fn parse_expectation_type_str(s: &str) -> Option<ExpectationType> {
    match s {
        "NotNull" => Some(ExpectationType::NotNull),
        "Unique" => Some(ExpectationType::Unique),
        "Between" => Some(ExpectationType::Between(0.0, 0.0)),
        "MatchRegex" => Some(ExpectationType::MatchRegex(String::new())),
        "NotMatchRegex" => Some(ExpectationType::NotMatchRegex(String::new())),
        "RowCountBetween" => Some(ExpectationType::RowCountBetween(0, 0)),
        "ColumnMeanBetween" => Some(ExpectationType::ColumnMeanBetween(0.0, 0.0)),
        "ColumnStddevBetween" => Some(ExpectationType::ColumnStddevBetween(0.0, 0.0)),
        "ColumnMinBetween" => Some(ExpectationType::ColumnMinBetween(0.0, 0.0)),
        "ColumnMaxBetween" => Some(ExpectationType::ColumnMaxBetween(0.0, 0.0)),
        "DistinctValuesEqualSet" => Some(ExpectationType::DistinctValuesEqualSet(vec![])),
        "DistinctValuesContainedInSet" => Some(ExpectationType::DistinctValuesContainedInSet(vec![])),
        "ColumnValuesToBeInSet" => Some(ExpectationType::ColumnValuesToBeInSet(vec![])),
        "ColumnKLDivergenceLessThan" => Some(ExpectationType::ColumnKLDivergenceLessThan(0.0)),
        "ColumnQuantileBetween" => Some(ExpectationType::ColumnQuantileBetween(0.0, 0.0, 0.0)),
        "TableColumnsMatchOrderedList" => Some(ExpectationType::TableColumnsMatchOrderedList(vec![])),
        _ => None,
    }
}
