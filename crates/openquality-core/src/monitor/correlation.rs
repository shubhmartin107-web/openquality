use crate::error::{OpenQualityError, Result};
use crate::types::*;
use chrono::Utc;
use std::collections::HashMap;

pub struct CorrelationMonitor;

impl CorrelationMonitor {
    pub fn check(
        values_x: &[f64],
        values_y: &[f64],
        method: &CorrelationMethod,
        threshold: f64,
    ) -> Result<MonitorResult> {
        if values_x.len() != values_y.len() || values_x.len() < 3 {
            return Err(OpenQualityError::Monitor(
                "Correlation requires equal-length arrays with at least 3 points".into(),
            ));
        }

        let observed = match method {
            CorrelationMethod::Pearson => Self::pearson(values_x, values_y),
            CorrelationMethod::Spearman => Self::spearman(values_x, values_y),
        };

        let alert = observed.abs() < threshold.abs();
        let mut details = HashMap::new();
        details.insert("correlation".into(), serde_json::json!(observed));
        details.insert("threshold".into(), serde_json::json!(threshold));
        details.insert("method".into(), serde_json::json!(format!("{:?}", method)));
        details.insert("n".into(), serde_json::json!(values_x.len()));

        Ok(MonitorResult {
            monitor_id: "correlation".into(),
            monitor_type: MonitorType::Correlation {
                column_x: String::new(),
                column_y: String::new(),
                method: method.clone(),
                threshold,
            },
            table_name: String::new(),
            alert,
            severity: if alert {
                Severity::Warning
            } else {
                Severity::Info
            },
            score: observed,
            threshold,
            message: if alert {
                format!(
                    "Correlation alert: {:?}={:.4} below threshold {:.4}",
                    method, observed, threshold
                )
            } else {
                format!("Correlation OK: {:?}={:.4}", method, observed)
            },
            details,
            timestamp: Utc::now(),
        })
    }

    fn pearson(x: &[f64], y: &[f64]) -> f64 {
        let n = x.len() as f64;
        let mean_x = x.iter().sum::<f64>() / n;
        let mean_y = y.iter().sum::<f64>() / n;
        let cov: f64 = x
            .iter()
            .zip(y)
            .map(|(xi, yi)| (xi - mean_x) * (yi - mean_y))
            .sum();
        let var_x: f64 = x.iter().map(|v| (v - mean_x).powi(2)).sum();
        let var_y: f64 = y.iter().map(|v| (v - mean_y).powi(2)).sum();
        let denom = (var_x * var_y).sqrt().max(1e-10);
        cov / denom
    }

    fn spearman(x: &[f64], y: &[f64]) -> f64 {
        let rank_x = Self::rank(x);
        let rank_y = Self::rank(y);
        Self::pearson(&rank_x, &rank_y)
    }

    fn rank(values: &[f64]) -> Vec<f64> {
        let mut sorted: Vec<(usize, f64)> = values.iter().copied().enumerate().collect();
        sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let mut ranks = vec![0.0; values.len()];
        for (i, (idx, _)) in sorted.iter().enumerate() {
            ranks[*idx] = i as f64 + 1.0;
        }
        ranks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pearson_perfect() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let r = CorrelationMonitor::pearson(&x, &y);
        assert!((r - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_pearson_negative() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        let r = CorrelationMonitor::pearson(&x, &y);
        assert!((r + 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_spearman() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        let r = CorrelationMonitor::spearman(&x, &y);
        assert!((r + 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_check_no_alert() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = CorrelationMonitor::check(&x, &y, &CorrelationMethod::Pearson, 0.99).unwrap();
        assert!(!result.alert);
    }

    #[test]
    fn test_check_alert_weak() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![100.0, 200.0, 300.0, 400.0, 500.0];
        let result = CorrelationMonitor::check(&x, &y, &CorrelationMethod::Pearson, 0.99).unwrap();
        assert!(!result.alert);
    }

    #[test]
    fn test_check_alert_uncorrelated() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![100.0, 1.0, 500.0, 3.0, 200.0];
        let result = CorrelationMonitor::check(&x, &y, &CorrelationMethod::Pearson, 0.8).unwrap();
        assert!(result.alert);
    }
}
