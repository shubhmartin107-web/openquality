use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExpectationType {
    NotNull,
    Unique,
    Between(f64, f64),
    MatchRegex(String),
    NotMatchRegex(String),
    DistinctValuesEqualSet(Vec<String>),
    DistinctValuesContainedInSet(Vec<String>),
    RowCountBetween(u64, u64),
    ColumnMeanBetween(f64, f64),
    ColumnStddevBetween(f64, f64),
    ColumnMinBetween(f64, f64),
    ColumnMaxBetween(f64, f64),
    ColumnValuesToBeInSet(Vec<String>),
    ColumnKLDivergenceLessThan(f64),
    ColumnQuantileBetween(f64, f64, f64),
    TableColumnsMatchOrderedList(Vec<String>),
    Custom(String, serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expectation {
    pub id: Uuid,
    pub expectation_type: ExpectationType,
    pub column: Option<String>,
    pub kwargs: HashMap<String, serde_json::Value>,
    pub meta: HashMap<String, String>,
    pub tolerance: f64,
}

impl Expectation {
    pub fn new(expectation_type: ExpectationType, column: Option<&str>) -> Self {
        Self {
            id: Uuid::new_v4(),
            expectation_type,
            column: column.map(String::from),
            kwargs: HashMap::new(),
            meta: HashMap::new(),
            tolerance: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectationSuite {
    pub name: String,
    pub expectations: Vec<Expectation>,
    pub meta: HashMap<String, String>,
}

impl ExpectationSuite {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            expectations: Vec::new(),
            meta: HashMap::new(),
        }
    }

    pub fn add(&mut self, expectation: Expectation) -> &mut Self {
        self.expectations.push(expectation);
        self
    }

    pub fn with(mut self, expectation: Expectation) -> Self {
        self.expectations.push(expectation);
        self
    }

    pub fn with_meta(mut self, key: &str, value: &str) -> Self {
        self.meta.insert(key.to_string(), value.to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectationResult {
    pub expectation_id: Uuid,
    pub expectation_type: ExpectationType,
    pub column: Option<String>,
    pub success: bool,
    pub observed_value: serde_json::Value,
    pub expected_value: serde_json::Value,
    pub details: HashMap<String, serde_json::Value>,
    pub exception_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteResult {
    pub suite_name: String,
    pub results: Vec<ExpectationResult>,
    pub success: bool,
    pub run_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub summary: SuiteSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub success_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MonitorType {
    Freshness {
        max_age_seconds: u64,
    },
    Volume {
        baseline_period_seconds: u64,
    },
    Schema,
    Distribution {
        metric: DistributionMetric,
        column: Option<String>,
    },
    Correlation {
        column_x: String,
        column_y: String,
        method: CorrelationMethod,
        threshold: f64,
    },
    Uniqueness {
        columns: Vec<String>,
        threshold_ratio: f64,
    },
    ReferentialIntegrity {
        source_column: String,
        target_table: String,
        target_column: String,
    },
    CustomSQL {
        query: String,
        expected: Option<f64>,
        comparison: ComparisonOp,
    },
    MLDrift {
        model_name: String,
        metric: MLDriftMetric,
        threshold: f64,
    },
    Cost {
        resource_type: CostResourceType,
        budget: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CorrelationMethod {
    Pearson,
    Spearman,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComparisonOp {
    GreaterThan,
    LessThan,
    EqualTo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MLDriftMetric {
    PredictionDrift,
    FeatureDrift,
    AccuracyDrop,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CostResourceType {
    QueryCost,
    StorageCost,
    ComputeCost,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DistributionMetric {
    KSTest,
    JSDivergence,
    ChiSquared,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThresholdMethod {
    ThreeSigma,
    IQR,
    Mad,
    Fixed(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    pub method: ThresholdMethod,
    pub sensitivity: f64,
    pub min_threshold: f64,
    pub max_threshold: f64,
    pub window_size: usize,
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            method: ThresholdMethod::Mad,
            sensitivity: 3.0,
            min_threshold: 0.001,
            max_threshold: f64::MAX,
            window_size: 20,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub id: String,
    pub monitor_type: MonitorType,
    pub table_name: String,
    pub auto_threshold: bool,
    pub threshold_config: ThresholdConfig,
    pub schedule_seconds: Option<u64>,
    pub enabled: bool,
    pub meta: HashMap<String, String>,
}

impl MonitorConfig {
    pub fn new(id: &str, monitor_type: MonitorType, table_name: &str) -> Self {
        Self {
            id: id.to_string(),
            monitor_type,
            table_name: table_name.to_string(),
            auto_threshold: true,
            threshold_config: ThresholdConfig::default(),
            schedule_seconds: None,
            enabled: true,
            meta: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorResult {
    pub monitor_id: String,
    pub monitor_type: MonitorType,
    pub table_name: String,
    pub alert: bool,
    pub severity: Severity,
    pub score: f64,
    pub threshold: f64,
    pub message: String,
    pub details: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Warning => write!(f, "WARNING"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub monitor_id: String,
    pub monitor_result: MonitorResult,
    pub severity: Severity,
    pub message: String,
    pub detail: serde_json::Value,
    pub root_cause_hints: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
    pub acked: bool,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub group_key: Option<String>,
    pub escalation_level: u32,
    pub owner_id: Option<Uuid>,
    pub snoozed_until: Option<DateTime<Utc>>,
}

impl Incident {
    pub fn new(monitor_result: MonitorResult, hints: Vec<String>) -> Self {
        let severity = monitor_result.severity.clone();
        let message = monitor_result.message.clone();
        let detail = serde_json::to_value(&monitor_result).unwrap_or_default();
        Self {
            id: Uuid::new_v4(),
            workspace_id: None,
            monitor_id: monitor_result.monitor_id.clone(),
            monitor_result,
            severity,
            message,
            detail,
            root_cause_hints: hints,
            timestamp: Utc::now(),
            resolved: false,
            resolved_at: None,
            acked: false,
            acknowledged_at: None,
            group_key: None,
            escalation_level: 0,
            owner_id: None,
            snoozed_until: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreRule {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub monitor_id: Option<String>,
    pub pattern: Option<String>,
    pub reason: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceWindow {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub monitor_id: Option<String>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub source_type: DataSourceType,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSourceType {
    Csv,
    Parquet,
    Pandas,
    Inline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnProfile {
    pub name: String,
    pub dtype: String,
    pub null_count: usize,
    pub distinct_count: usize,
    pub row_count: usize,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub mean: Option<f64>,
    pub stddev: Option<f64>,
    pub quantiles: Option<[f64; 5]>,
}
