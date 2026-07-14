use crate::error::Result;
use crate::types::*;
use chrono::Utc;
use std::collections::HashMap;

pub struct CustomSQLMonitor;

impl CustomSQLMonitor {
    pub fn check(
        observed: f64,
        expected: Option<f64>,
        comparison: &ComparisonOp,
    ) -> Result<MonitorResult> {
        let alert = match (comparison, expected) {
            (ComparisonOp::GreaterThan, Some(exp)) => observed > exp,
            (ComparisonOp::LessThan, Some(exp)) => observed < exp,
            (ComparisonOp::EqualTo, Some(exp)) => (observed - exp).abs() > 1e-6,
            _ => false,
        };

        let mut details = HashMap::new();
        details.insert("observed".into(), serde_json::json!(observed));
        details.insert("expected".into(), serde_json::json!(expected));
        details.insert(
            "comparison".into(),
            serde_json::json!(format!("{:?}", comparison)),
        );

        Ok(MonitorResult {
            monitor_id: "custom_sql".into(),
            monitor_type: MonitorType::CustomSQL {
                query: String::new(),
                expected,
                comparison: comparison.clone(),
            },
            table_name: String::new(),
            alert,
            severity: if alert {
                Severity::Warning
            } else {
                Severity::Info
            },
            score: observed,
            threshold: expected.unwrap_or(0.0),
            message: if alert {
                format!(
                    "Custom SQL alert: observed={:.4} {:?} expected={:?}",
                    observed, comparison, expected
                )
            } else {
                format!("Custom SQL OK: observed={:.4}", observed)
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
    fn test_greater_than_alert() {
        let result =
            CustomSQLMonitor::check(100.0, Some(50.0), &ComparisonOp::GreaterThan).unwrap();
        assert!(result.alert);
    }

    #[test]
    fn test_greater_than_ok() {
        let result = CustomSQLMonitor::check(30.0, Some(50.0), &ComparisonOp::GreaterThan).unwrap();
        assert!(!result.alert);
    }

    #[test]
    fn test_less_than_alert() {
        let result = CustomSQLMonitor::check(10.0, Some(50.0), &ComparisonOp::LessThan).unwrap();
        assert!(result.alert);
    }

    #[test]
    fn test_equal_to() {
        let result = CustomSQLMonitor::check(42.0, Some(42.0), &ComparisonOp::EqualTo).unwrap();
        assert!(!result.alert);
    }

    #[test]
    fn test_no_expected() {
        let result = CustomSQLMonitor::check(42.0, None, &ComparisonOp::GreaterThan).unwrap();
        assert!(!result.alert);
    }
}
