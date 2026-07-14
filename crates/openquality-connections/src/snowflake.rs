use async_trait::async_trait;
use openquality_core::error::{OpenQualityError, Result};
use polars::prelude::*;

use crate::sources::*;

pub struct SnowflakeConnector {
    config: ConnectorConfig,
}

impl SnowflakeConnector {
    pub fn new(config: ConnectorConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl DataSourceConnector for SnowflakeConnector {
    fn name(&self) -> &str {
        "snowflake"
    }

    async fn test_connection(&self) -> Result<bool> {
        tracing::info!(
            "Snowflake connection test (stub) for account={:?}",
            self.config.host
        );
        Ok(true)
    }

    async fn list_tables(&self, _schema: Option<&str>) -> Result<Vec<TableInfo>> {
        tracing::info!("Snowflake list_tables (stub)");
        Ok(Vec::new())
    }

    async fn get_table_info(&self, _table: &str, _schema: Option<&str>) -> Result<TableInfo> {
        Err(OpenQualityError::Connection(
            "Snowflake connector: stub — real implementation requires snowflake-api crate".into(),
        ))
    }

    async fn execute_query(&self, _query: &str) -> Result<QueryResult> {
        Err(OpenQualityError::Connection(
            "Snowflake connector: stub".into(),
        ))
    }

    async fn fetch_dataframe(
        &self,
        _table: &str,
        _schema: Option<&str>,
        _limit: Option<i64>,
    ) -> Result<DataFrame> {
        Err(OpenQualityError::Connection(
            "Snowflake connector: stub".into(),
        ))
    }
}
