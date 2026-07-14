//! # OpenQuality Integrations
//!
//! Integration modules for external data tools.
//!
//! - [`dbt`] — Parse `manifest.json` to extract models, sources, tests, and lineage edges
//! - [`airflow`] — Parse Airflow webhook payloads (DAG run and task instance events)
//! - [`ge`] — Translate Great Expectations suite JSON to OpenQuality expectation types (16+ mappings)
//! - [`lineage`] — SQL parser for column-level lineage extraction and table dependency graphs

pub mod airflow;
pub mod dbt;
pub mod ge;
pub mod lineage;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum IntegrationError {
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Store error: {0}")]
    Store(String),
    #[error("Not found: {0}")]
    NotFound(String),
}
