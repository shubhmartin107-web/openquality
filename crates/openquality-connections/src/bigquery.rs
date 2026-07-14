use async_trait::async_trait;
use openquality_core::error::{OpenQualityError, Result};
use polars::prelude::*;

use crate::sources::*;

pub struct BigQueryConnector {
    config: ConnectorConfig,
}

impl BigQueryConnector {
    pub fn new(config: ConnectorConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl DataSourceConnector for BigQueryConnector {
    fn name(&self) -> &str {
        "bigquery"
    }

    async fn test_connection(&self) -> Result<bool> {
        tracing::info!(
            "BigQuery connection test (stub) for project={:?}",
            self.config.project_id
        );
        Ok(true)
    }

    async fn list_tables(&self, _schema: Option<&str>) -> Result<Vec<TableInfo>> {
        tracing::info!("BigQuery list_tables (stub)");
        Ok(Vec::new())
    }

    async fn get_table_info(&self, _table: &str, _schema: Option<&str>) -> Result<TableInfo> {
        Err(OpenQualityError::Connection(
            "BigQuery connector: stub — real implementation requires gcloud-sdk crate".into(),
        ))
    }

    async fn execute_query(&self, _query: &str) -> Result<QueryResult> {
        Err(OpenQualityError::Connection(
            "BigQuery connector: stub".into(),
        ))
    }

    async fn fetch_dataframe(
        &self,
        _table: &str,
        _schema: Option<&str>,
        _limit: Option<i64>,
    ) -> Result<DataFrame> {
        Err(OpenQualityError::Connection(
            "BigQuery connector: stub".into(),
        ))
    }
}
