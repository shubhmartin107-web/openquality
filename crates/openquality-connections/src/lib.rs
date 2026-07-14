//! # OpenQuality Connections
//!
//! Data source connectors for profiling and expectation execution.
//!
//! - [`PostgresConnector`] — PostgreSQL connection via sqlx (full implementation)
//! - [`SnowflakeConnector`] — Snowflake stub connector
//! - [`BigQueryConnector`] — Google BigQuery stub connector
//! - [`sources`] — Data source registry and connection management

pub mod bigquery;
pub mod postgres;
pub mod snowflake;
pub mod sources;

pub use bigquery::BigQueryConnector;
pub use postgres::PostgresConnector;
pub use snowflake::SnowflakeConnector;
