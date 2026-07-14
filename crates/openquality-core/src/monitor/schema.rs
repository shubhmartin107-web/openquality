use crate::error::Result;
use crate::types::*;
use chrono::Utc;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SchemaSnapshot {
    pub columns: Vec<ColumnInfo>,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub dtype: String,
    pub nullable: bool,
}

pub struct SchemaMonitor {
    pub config: MonitorConfig,
    pub last_snapshot: Option<SchemaSnapshot>,
}

impl SchemaMonitor {
    pub fn new(config: MonitorConfig) -> Self {
        Self {
            config,
            last_snapshot: None,
        }
    }

    pub fn check(&mut self, df: &DataFrame) -> Result<MonitorResult> {
        let current = Self::snapshot(df);
        let mut details = HashMap::new();
        let alert = if let Some(ref last) = self.last_snapshot {
            let changes = SchemaMonitor::diff(last, &current);
            details.insert("changes".into(), serde_json::json!(changes));
            if !changes.is_empty() {
                details.insert(
                    "added_columns".into(),
                    serde_json::json!(
                        changes
                            .iter()
                            .filter(|c| c.starts_with("+"))
                            .collect::<Vec<_>>()
                    ),
                );
                details.insert(
                    "removed_columns".into(),
                    serde_json::json!(
                        changes
                            .iter()
                            .filter(|c| c.starts_with("-"))
                            .collect::<Vec<_>>()
                    ),
                );
                details.insert(
                    "changed_columns".into(),
                    serde_json::json!(
                        changes
                            .iter()
                            .filter(|c| c.starts_with("~"))
                            .collect::<Vec<_>>()
                    ),
                );
            }
            !changes.is_empty()
        } else {
            false
        };
        self.last_snapshot = Some(current.clone());
        Ok(MonitorResult {
            monitor_id: self.config.id.clone(),
            monitor_type: self.config.monitor_type.clone(),
            table_name: self.config.table_name.clone(),
            alert,
            severity: Severity::Warning,
            score: if alert { 1.0 } else { 0.0 },
            threshold: 0.0,
            message: if alert {
                let changes = SchemaMonitor::diff(self.last_snapshot.as_ref().unwrap(), &current);
                format!(
                    "Schema alert: table '{}' schema changed — {:?}",
                    self.config.table_name, changes
                )
            } else {
                format!(
                    "Schema OK: table '{}' schema unchanged",
                    self.config.table_name
                )
            },
            details,
            timestamp: Utc::now(),
        })
    }

    fn snapshot(df: &DataFrame) -> SchemaSnapshot {
        let columns: Vec<ColumnInfo> = df
            .get_column_names()
            .iter()
            .map(|name| {
                let series = df.column(name).ok();
                let dtype = series
                    .map(|s| format!("{:?}", s.dtype()))
                    .unwrap_or_default();
                ColumnInfo {
                    name: name.to_string(),
                    dtype,
                    nullable: true,
                }
            })
            .collect();
        let hash = format!(
            "{:?}",
            columns
                .iter()
                .map(|c| format!("{}:{}", c.name, c.dtype))
                .collect::<Vec<_>>()
        );
        SchemaSnapshot { columns, hash }
    }

    fn diff(before: &SchemaSnapshot, after: &SchemaSnapshot) -> Vec<String> {
        let mut changes = Vec::new();
        let before_map: HashMap<&str, &ColumnInfo> = before
            .columns
            .iter()
            .map(|c| (c.name.as_str(), c))
            .collect();
        let after_map: HashMap<&str, &ColumnInfo> =
            after.columns.iter().map(|c| (c.name.as_str(), c)).collect();
        for col in &after.columns {
            if !before_map.contains_key(col.name.as_str()) {
                changes.push(format!("+{} ({})", col.name, col.dtype));
            } else if let Some(b) = before_map.get(col.name.as_str()) {
                if b.dtype != col.dtype {
                    changes.push(format!("~{}: {} -> {}", col.name, b.dtype, col.dtype));
                }
            }
        }
        for col in &before.columns {
            if !after_map.contains_key(col.name.as_str()) {
                changes.push(format!("-{} ({})", col.name, col.dtype));
            }
        }
        changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> MonitorConfig {
        MonitorConfig::new("schema_test", MonitorType::Schema, "test_table")
    }

    fn test_df() -> DataFrame {
        df!(
            "a" => &[1.0f64, 2.0],
            "b" => &[3.0f64, 4.0],
        )
        .unwrap()
    }

    #[test]
    fn test_column_added() {
        let mut monitor = SchemaMonitor::new(config());
        let df1 = test_df();
        let _ = monitor.check(&df1).unwrap();
        let df2 = df!(
            "a" => &[1.0f64, 2.0],
            "b" => &[3.0f64, 4.0],
            "c" => &[5.0f64, 6.0],
        )
        .unwrap();
        let result = monitor.check(&df2).unwrap();
        assert!(result.alert);
        let changes: Vec<String> =
            serde_json::from_value(result.details["changes"].clone()).unwrap();
        assert!(changes.iter().any(|c| c.starts_with('+')));
    }

    #[test]
    fn test_column_removed() {
        let mut monitor = SchemaMonitor::new(config());
        let df1 = test_df();
        let _ = monitor.check(&df1).unwrap();
        let df2 = df!("a" => &[1.0f64, 2.0]).unwrap();
        let result = monitor.check(&df2).unwrap();
        assert!(result.alert);
        let changes: Vec<String> =
            serde_json::from_value(result.details["changes"].clone()).unwrap();
        assert!(changes.iter().any(|c| c.starts_with('-')));
    }

    #[test]
    fn test_schema_unchanged() {
        let mut monitor = SchemaMonitor::new(config());
        let df = test_df();
        let _ = monitor.check(&df).unwrap();
        let result = monitor.check(&df).unwrap();
        assert!(!result.alert);
    }

    #[test]
    fn test_column_type_changed() {
        let mut monitor = SchemaMonitor::new(config());
        let df1 = test_df();
        let _ = monitor.check(&df1).unwrap();
        let df2 = df!(
            "a" => &["x", "y"],
            "b" => &[3.0f64, 4.0],
        )
        .unwrap();
        let result = monitor.check(&df2).unwrap();
        assert!(result.alert);
    }

    #[test]
    fn test_first_check_no_alert() {
        let mut monitor = SchemaMonitor::new(config());
        let df = test_df();
        let result = monitor.check(&df).unwrap();
        assert!(!result.alert);
    }
}
