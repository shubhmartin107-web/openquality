use async_trait::async_trait;
use openquality_core::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub schema: String,
    pub table_name: String,
    pub columns: Vec<ColumnInfo>,
    pub row_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub dtype: String,
    pub nullable: bool,
    pub is_pk: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: i64,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorConfig {
    pub source_type: String,
    pub connection_string: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: Option<String>,
    pub schema: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub project_id: Option<String>,
    pub dataset_id: Option<String>,
    pub extra: HashMap<String, String>,
}

#[async_trait]
pub trait DataSourceConnector: Send + Sync {
    fn name(&self) -> &str;

    async fn test_connection(&self) -> Result<bool>;

    async fn list_tables(&self, schema: Option<&str>) -> Result<Vec<TableInfo>>;

    async fn get_table_info(&self, table: &str, schema: Option<&str>) -> Result<TableInfo>;

    async fn execute_query(&self, query: &str) -> Result<QueryResult>;

    async fn fetch_dataframe(
        &self,
        table: &str,
        schema: Option<&str>,
        limit: Option<i64>,
    ) -> Result<polars::prelude::DataFrame>;
}
