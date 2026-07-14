use crate::error::Result;
use crate::types::*;
use chrono::Utc;
use std::collections::HashMap;

pub struct ReferentialIntegrityMonitor;

impl ReferentialIntegrityMonitor {
    pub fn check(
        source_values: &[&str],
        target_values: &std::collections::HashSet<&str>,
    ) -> Result<MonitorResult> {
        let total = source_values.len();
        let orphan_count = source_values
            .iter()
            .filter(|v| !target_values.contains(*v))
            .count();
        let orphan_ratio = if total > 0 {
            orphan_count as f64 / total as f64
        } else {
            0.0
        };

        let alert = orphan_count > 0;
        let mut details = HashMap::new();
        details.insert("total_rows".into(), serde_json::json!(total));
        details.insert("orphan_count".into(), serde_json::json!(orphan_count));
        details.insert("orphan_ratio".into(), serde_json::json!(orphan_ratio));

        Ok(MonitorResult {
            monitor_id: "referential".into(),
            monitor_type: MonitorType::ReferentialIntegrity {
                source_column: String::new(),
                target_table: String::new(),
                target_column: String::new(),
            },
            table_name: String::new(),
            alert,
            severity: if alert {
                Severity::Critical
            } else {
                Severity::Info
            },
            score: orphan_count as f64,
            threshold: 0.0,
            message: if alert {
                format!(
                    "Referential integrity alert: {} orphan rows ({:.1}%)",
                    orphan_count,
                    orphan_ratio * 100.0
                )
            } else {
                "Referential integrity OK: no orphan rows".into()
            },
            details,
            timestamp: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_orphans() {
        let source = vec!["a", "b", "c"];
        let target: std::collections::HashSet<&str> = ["a", "b", "c", "d"].into();
        let result = ReferentialIntegrityMonitor::check(&source, &target).unwrap();
        assert!(!result.alert);
    }

    #[test]
    fn test_with_orphans() {
        let source = vec!["a", "b", "x", "y"];
        let target: std::collections::HashSet<&str> = ["a", "b", "c"].into();
        let result = ReferentialIntegrityMonitor::check(&source, &target).unwrap();
        assert!(result.alert);
        assert_eq!(result.score, 2.0);
    }

    #[test]
    fn test_empty_source() {
        let source: Vec<&str> = vec![];
        let target: std::collections::HashSet<&str> = ["a"].into();
        let result = ReferentialIntegrityMonitor::check(&source, &target).unwrap();
        assert!(!result.alert);
    }
}
