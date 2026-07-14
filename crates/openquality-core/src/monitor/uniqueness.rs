use crate::error::Result;
use crate::types::*;
use chrono::Utc;
use std::collections::HashMap;

pub struct UniquenessMonitor;

impl UniquenessMonitor {
    pub fn check(values: &[&str], threshold_ratio: f64) -> Result<MonitorResult> {
        let total = values.len();
        let unique: std::collections::HashSet<&str> = values.iter().copied().collect();
        let duplicate_count = total.saturating_sub(unique.len());
        let dup_ratio = if total > 0 {
            duplicate_count as f64 / total as f64
        } else {
            0.0
        };

        let alert = dup_ratio > threshold_ratio;
        let mut details = HashMap::new();
        details.insert("total_rows".into(), serde_json::json!(total));
        details.insert("unique_values".into(), serde_json::json!(unique.len()));
        details.insert("duplicate_count".into(), serde_json::json!(duplicate_count));
        details.insert("duplicate_ratio".into(), serde_json::json!(dup_ratio));
        details.insert("threshold_ratio".into(), serde_json::json!(threshold_ratio));

        Ok(MonitorResult {
            monitor_id: "uniqueness".into(),
            monitor_type: MonitorType::Uniqueness {
                columns: vec![],
                threshold_ratio,
            },
            table_name: String::new(),
            alert,
            severity: if alert {
                Severity::Warning
            } else {
                Severity::Info
            },
            score: dup_ratio,
            threshold: threshold_ratio,
            message: if alert {
                format!(
                    "Uniqueness alert: {:.1}% duplicates (threshold: {:.1}%)",
                    dup_ratio * 100.0,
                    threshold_ratio * 100.0
                )
            } else {
                format!("Uniqueness OK: {:.1}% duplicates", dup_ratio * 100.0)
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
    fn test_all_unique() {
        let v = vec!["a", "b", "c", "d"];
        let result = UniquenessMonitor::check(&v, 0.1).unwrap();
        assert!(!result.alert);
        assert!((result.score - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_duplicates() {
        let v = vec!["a", "a", "a", "b"];
        let result = UniquenessMonitor::check(&v, 0.1).unwrap();
        assert!(result.alert);
        assert!((result.score - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_empty() {
        let v: Vec<&str> = vec![];
        let result = UniquenessMonitor::check(&v, 0.1).unwrap();
        assert!(!result.alert);
    }
}
