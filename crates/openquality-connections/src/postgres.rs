use async_trait::async_trait;
use openquality_core::error::{OpenQualityError, Result};
use polars::prelude::*;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

use crate::sources::*;

fn validate_identifier(name: &str) -> Result<()> {
    if name.is_empty()
        || name.len() > 63
        || !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '$')
    {
        return Err(OpenQualityError::Connection(format!(
            "Invalid SQL identifier: {name}"
        )));
    }
    Ok(())
}

pub struct PostgresConnector {
    config: ConnectorConfig,
}

impl PostgresConnector {
    pub fn new(config: ConnectorConfig) -> Self {
        Self { config }
    }

    fn conn_string(&self) -> String {
        if let Some(ref cs) = self.config.connection_string {
            return cs.clone();
        }
        let host = self.config.host.as_deref().unwrap_or("localhost");
        let port = self.config.port.unwrap_or(5432);
        let db = self.config.database.as_deref().unwrap_or("postgres");
        let user = self.config.username.as_deref().unwrap_or("postgres");
        let pass = self.config.password.as_deref().unwrap_or("");
        format!("postgres://{user}:{pass}@{host}:{port}/{db}")
    }

    async fn pool(&self) -> std::result::Result<PgPool, sqlx::Error> {
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&self.conn_string())
            .await
    }
}

#[async_trait]
impl DataSourceConnector for PostgresConnector {
    fn name(&self) -> &str {
        "postgresql"
    }

    async fn test_connection(&self) -> Result<bool> {
        let pool = self
            .pool()
            .await
            .map_err(|e| OpenQualityError::Connection(e.to_string()))?;
        let ok = sqlx::query("SELECT 1").execute(&pool).await.is_ok();
        pool.close().await;
        Ok(ok)
    }

    async fn list_tables(&self, schema: Option<&str>) -> Result<Vec<TableInfo>> {
        let pool = self
            .pool()
            .await
            .map_err(|e| OpenQualityError::Connection(e.to_string()))?;
        let schema_filter = schema.unwrap_or("public");
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT table_schema, table_name FROM information_schema.tables WHERE table_schema = $1 AND table_type = 'BASE TABLE'",
        )
        .bind(schema_filter)
        .fetch_all(&pool)
        .await
        .map_err(|e| OpenQualityError::Connection(e.to_string()))?;
        let mut tables = Vec::new();
        for (schema, table_name) in rows {
            let info = self.get_table_info(&table_name, Some(&schema)).await?;
            tables.push(info);
        }
        pool.close().await;
        Ok(tables)
    }

    async fn get_table_info(&self, table: &str, schema: Option<&str>) -> Result<TableInfo> {
        let pool = self
            .pool()
            .await
            .map_err(|e| OpenQualityError::Connection(e.to_string()))?;
        let schema_filter = schema.unwrap_or("public");
        let columns: Vec<(String, String, bool)> = sqlx::query_as(
            "SELECT column_name, data_type, is_nullable = 'YES' \
             FROM information_schema.columns \
             WHERE table_schema = $1 AND table_name = $2 \
             ORDER BY ordinal_position",
        )
        .bind(schema_filter)
        .bind(table)
        .fetch_all(&pool)
        .await
        .map_err(|e| OpenQualityError::Connection(e.to_string()))?;
        let col_infos: Vec<ColumnInfo> = columns
            .into_iter()
            .map(|(name, dtype, nullable)| ColumnInfo {
                name,
                dtype,
                nullable,
                is_pk: false,
            })
            .collect();
        validate_identifier(schema_filter)?;
        validate_identifier(table)?;
        let row_count: Option<i64> = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM \"{}\".\"{}\"",
            schema_filter, table
        ))
        .fetch_one(&pool)
        .await
        .ok();
        pool.close().await;
        Ok(TableInfo {
            schema: schema_filter.to_string(),
            table_name: table.to_string(),
            columns: col_infos,
            row_count,
        })
    }

    async fn execute_query(&self, query: &str) -> Result<QueryResult> {
        let pool = self
            .pool()
            .await
            .map_err(|e| OpenQualityError::Connection(e.to_string()))?;
        let start = std::time::Instant::now();
        let rows = sqlx::query(query)
            .fetch_all(&pool)
            .await
            .map_err(|e| OpenQualityError::Connection(e.to_string()))?;
        let elapsed = start.elapsed().as_millis() as u64;
        pool.close().await;
        Ok(QueryResult {
            columns: Vec::new(),
            rows: Vec::new(),
            row_count: rows.len() as i64,
            execution_time_ms: elapsed,
        })
    }

    async fn fetch_dataframe(
        &self,
        table: &str,
        schema: Option<&str>,
        limit: Option<i64>,
    ) -> Result<DataFrame> {
        let pool = self
            .pool()
            .await
            .map_err(|e| OpenQualityError::Connection(e.to_string()))?;
        let schema_filter = schema.unwrap_or("public");
        validate_identifier(schema_filter)?;
        validate_identifier(table)?;
        let query = match limit {
            Some(l) => format!(
                "SELECT * FROM \"{}\".\"{}\" LIMIT {}",
                schema_filter, table, l
            ),
            None => format!("SELECT * FROM \"{}\".\"{}\"", schema_filter, table),
        };
        let rows = sqlx::query(&query)
            .fetch_all(&pool)
            .await
            .map_err(|e| OpenQualityError::Connection(e.to_string()))?;
        pool.close().await;

        if rows.is_empty() {
            return Ok(DataFrame::empty());
        }

        let mut cols: Vec<Column> = Vec::new();
        let column_count = rows[0].len();

        for i in 0..column_count {
            let name = format!("col_{}", i);

            if rows[0].try_get::<f64, usize>(i).is_ok() {
                let vals: Vec<Option<f64>> = rows
                    .iter()
                    .map(|r| r.try_get::<f64, usize>(i).ok())
                    .collect();
                cols.push(polars::prelude::Column::new(name.into(), &vals));
            } else if rows[0].try_get::<i64, usize>(i).is_ok() {
                let vals: Vec<Option<i64>> = rows
                    .iter()
                    .map(|r| r.try_get::<i64, usize>(i).ok())
                    .collect();
                cols.push(polars::prelude::Column::new(name.into(), &vals));
            } else if rows[0].try_get::<String, usize>(i).is_ok() {
                let vals: Vec<Option<String>> = rows
                    .iter()
                    .map(|r| r.try_get::<String, usize>(i).ok())
                    .collect();
                cols.push(polars::prelude::Column::new(name.into(), &vals));
            } else if rows[0].try_get::<bool, usize>(i).is_ok() {
                let vals: Vec<Option<bool>> = rows
                    .iter()
                    .map(|r| r.try_get::<bool, usize>(i).ok())
                    .collect();
                let str_vals: Vec<Option<String>> =
                    vals.into_iter().map(|v| v.map(|b| b.to_string())).collect();
                cols.push(polars::prelude::Column::new(name.into(), &str_vals));
            }
        }

        DataFrame::new(cols).map_err(OpenQualityError::Polars)
    }
}
