use crate::error::Result;
use crate::types::*;
use chrono::Utc;
use std::collections::HashMap;

pub struct MLDriftMonitor;

impl MLDriftMonitor {
    pub fn check(
        current_score: f64,
        model_name: &str,
        metric: &MLDriftMetric,
        threshold: f64,
    ) -> Result<MonitorResult> {
        let alert = current_score > threshold;
        let mut details = HashMap::new();
        details.insert("model_name".into(), serde_json::json!(model_name));
        details.insert(
            "drift_metric".into(),
            serde_json::json!(format!("{:?}", metric)),
        );
        details.insert("threshold".into(), serde_json::json!(threshold));

        Ok(MonitorResult {
            monitor_id: format!("ml_drift_{}", model_name),
            monitor_type: MonitorType::MLDrift {
                model_name: model_name.to_string(),
                metric: metric.clone(),
                threshold,
            },
            table_name: String::new(),
            alert,
            severity: if alert && current_score > threshold * 2.0 {
                Severity::Critical
            } else if alert {
                Severity::Warning
            } else {
                Severity::Info
            },
            score: current_score,
            threshold,
            message: if alert {
                format!(
                    "ML drift alert: model '{}' {:?}={:.4} (threshold={:.4})",
                    model_name, metric, current_score, threshold
                )
            } else {
                format!(
                    "ML drift OK: model '{}' {:?}={:.4}",
                    model_name, metric, current_score
                )
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
    fn test_no_drift() {
        let result =
            MLDriftMonitor::check(0.05, "my_model", &MLDriftMetric::PredictionDrift, 0.2).unwrap();
        assert!(!result.alert);
    }

    #[test]
    fn test_drift_alert() {
        let result =
            MLDriftMonitor::check(0.5, "my_model", &MLDriftMetric::FeatureDrift, 0.2).unwrap();
        assert!(result.alert);
    }

    #[test]
    fn test_drift_critical() {
        let result =
            MLDriftMonitor::check(0.9, "my_model", &MLDriftMetric::AccuracyDrop, 0.2).unwrap();
        assert!(result.alert);
        assert_eq!(result.severity, Severity::Critical);
    }
}
