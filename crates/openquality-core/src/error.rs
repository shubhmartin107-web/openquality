use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpenQualityError {
    #[error("Data source error: {0}")]
    DataSource(String),

    #[error("Invalid expectation: {0}")]
    InvalidExpectation(String),

    #[error("Monitor error: {0}")]
    Monitor(String),

    #[error("Statistical computation failed: {0}")]
    Stats(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Polars error: {0}")]
    Polars(#[from] polars::prelude::PolarsError),

    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),

    #[error("Parquet error: {0}")]
    Parquet(#[from] parquet::errors::ParquetError),

    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Serde YAML error: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Store error: {0}")]
    Store(String),

    #[error("Connection error: {0}")]
    Connection(String),
}

pub type Result<T> = std::result::Result<T, OpenQualityError>;
