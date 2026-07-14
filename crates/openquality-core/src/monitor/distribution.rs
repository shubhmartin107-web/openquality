use crate::error::Result;
use crate::stats::chi_square::chi_square_from_slices;
use crate::stats::js_divergence::js_divergence_samples;
use crate::stats::ks_test::ks_test;
use crate::types::*;
use chrono::Utc;
use std::collections::HashMap;

pub struct DistributionMonitor {
    pub config: MonitorConfig,
    pub reference_histogram: Option<Vec<f64>>,
    pub reference_values: Option<Vec<f64>>,
    pub reference_labels: Option<Vec<String>>,
    pub history: Vec<f64>,
}

impl DistributionMonitor {
    pub fn new(config: MonitorConfig) -> Self {
        Self {
            config,
            reference_histogram: None,
            reference_values: None,
            reference_labels: None,
            history: Vec::new(),
        }
    }

    pub fn set_reference_numeric(&mut self, values: &[f64]) {
        self.reference_values = Some(values.to_vec());
        let global_min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let global_max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        if let Ok(hist) =
            crate::stats::js_divergence::histogram_from(values, 20, global_min, global_max)
        {
            self.reference_histogram = Some(hist);
        }
    }

    pub fn set_reference_categorical(&mut self, labels: &[String]) {
        self.reference_labels = Some(labels.to_vec());
    }

    pub fn check_numeric(&mut self, target_values: &[f64]) -> Result<MonitorResult> {
        let ref_values = self.reference_values.as_deref().unwrap_or(target_values);
        let metric = match &self.config.monitor_type {
            MonitorType::Distribution { metric, .. } => metric.clone(),
            _ => DistributionMetric::KSTest,
        };
        let (score, threshold) = match metric {
            DistributionMetric::KSTest => {
                let result = ks_test(ref_values, target_values)?;
                let t = if self.config.auto_threshold && self.history.len() > 5 {
                    let mut h = self.history.clone();
                    h.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let med = h[h.len() / 2];
                    let mad: f64 = h.iter().map(|v| (v - med).abs()).sum::<f64>() / h.len() as f64;
                    med + self.config.threshold_config.sensitivity * 1.4826 * mad
                } else {
                    0.3
                };
                (result.statistic, t)
            }
            DistributionMetric::JSDivergence => {
                let bins = 20;
                let js = js_divergence_samples(ref_values, target_values, bins)?;
                let t = if self.config.auto_threshold && self.history.len() > 5 {
                    let mut h = self.history.clone();
                    h.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let med = h[h.len() / 2];
                    let mad: f64 = h.iter().map(|v| (v - med).abs()).sum::<f64>() / h.len() as f64;
                    med + self.config.threshold_config.sensitivity * 1.4826 * mad
                } else {
                    0.1
                };
                (js, t)
            }
            DistributionMetric::ChiSquared => {
                let ref_strs: Vec<String> = ref_values.iter().map(|v| v.to_string()).collect();
                let tgt_strs: Vec<String> = target_values.iter().map(|v| v.to_string()).collect();
                let ref_slices: Vec<&str> = ref_strs.iter().map(|s| s.as_str()).collect();
                let tgt_slices: Vec<&str> = tgt_strs.iter().map(|s| s.as_str()).collect();
                let result = chi_square_from_slices(&ref_slices, &tgt_slices).unwrap_or({
                    crate::stats::chi_square::ChiSquareResult {
                        statistic: 0.0,
                        degrees_of_freedom: 0,
                        critical_value: 3.841,
                        p_value: 1.0,
                        significant: false,
                    }
                });
                (result.statistic, 3.841)
            }
        };
        self.history.push(score);
        if self.history.len() > self.config.threshold_config.window_size {
            self.history.remove(0);
        }
        let alert = score > threshold;
        let severity = if alert && score > threshold * 2.0 {
            Severity::Critical
        } else {
            Severity::Warning
        };
        let mut details = HashMap::new();
        details.insert("metric".into(), serde_json::json!(format!("{:?}", metric)));
        details.insert("score".into(), serde_json::json!(score));
        details.insert("threshold".into(), serde_json::json!(threshold));
        details.insert(
            "reference_count".into(),
            serde_json::json!(ref_values.len()),
        );
        details.insert(
            "target_count".into(),
            serde_json::json!(target_values.len()),
        );
        Ok(MonitorResult {
            monitor_id: self.config.id.clone(),
            monitor_type: self.config.monitor_type.clone(),
            table_name: self.config.table_name.clone(),
            alert,
            severity,
            score,
            threshold,
            message: if alert {
                format!(
                    "Distribution drift detected on '{}': {:?} score={:.4} (threshold={:.4})",
                    self.config.table_name, metric, score, threshold
                )
            } else {
                format!(
                    "Distribution OK on '{}': {:?} score={:.4}",
                    self.config.table_name, metric, score
                )
            },
            details,
            timestamp: Utc::now(),
        })
    }

    pub fn check_categorical(&mut self, target_labels: &[String]) -> Result<MonitorResult> {
        let ref_labels = self.reference_labels.as_deref().unwrap_or(target_labels);
        let ref_strs: Vec<&str> = ref_labels.iter().map(|s| s.as_str()).collect();
        let tgt_strs: Vec<&str> = target_labels.iter().map(|s| s.as_str()).collect();
        let result = chi_square_from_slices(&ref_strs, &tgt_strs)?;
        let alert = result.significant;
        let mut details = HashMap::new();
        details.insert(
            "chi_square_statistic".into(),
            serde_json::json!(result.statistic),
        );
        details.insert("p_value".into(), serde_json::json!(result.p_value));
        details.insert(
            "degrees_of_freedom".into(),
            serde_json::json!(result.degrees_of_freedom),
        );
        Ok(MonitorResult {
            monitor_id: self.config.id.clone(),
            monitor_type: self.config.monitor_type.clone(),
            table_name: self.config.table_name.clone(),
            alert,
            severity: Severity::Warning,
            score: result.statistic,
            threshold: result.critical_value,
            message: if alert {
                format!(
                    "Categorical distribution drift on '{}': chi-sq={:.4}, p={:.4}",
                    self.config.table_name, result.statistic, result.p_value
                )
            } else {
                format!(
                    "Categorical distribution OK on '{}': chi-sq={:.4}",
                    self.config.table_name, result.statistic
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

    fn numeric_config(metric: DistributionMetric) -> MonitorConfig {
        let mut c = MonitorConfig::new(
            "dist_test",
            MonitorType::Distribution {
                metric,
                column: Some("col".into()),
            },
            "test_table",
        );
        c.auto_threshold = false;
        c
    }

    #[test]
    fn test_ks_test_no_drift() {
        let mut monitor = DistributionMonitor::new(numeric_config(DistributionMetric::KSTest));
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        monitor.set_reference_numeric(&data);
        let result = monitor.check_numeric(&data).unwrap();
        assert!(!result.alert);
        assert!(result.score < 0.3);
    }

    #[test]
    fn test_ks_test_drift() {
        let mut monitor = DistributionMonitor::new(numeric_config(DistributionMetric::KSTest));
        monitor.set_reference_numeric(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let result = monitor
            .check_numeric(&[100.0, 200.0, 300.0, 400.0, 500.0])
            .unwrap();
        assert!(result.alert);
        assert!(result.score > 0.3);
    }

    #[test]
    fn test_js_divergence_no_drift() {
        let mut monitor =
            DistributionMonitor::new(numeric_config(DistributionMetric::JSDivergence));
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        monitor.set_reference_numeric(&data);
        let result = monitor.check_numeric(&data).unwrap();
        assert!(!result.alert);
    }

    #[test]
    fn test_js_divergence_drift() {
        let mut monitor =
            DistributionMonitor::new(numeric_config(DistributionMetric::JSDivergence));
        monitor.set_reference_numeric(&vec![1.0; 100]);
        let result = monitor.check_numeric(&vec![100.0; 100]).unwrap();
        assert!(result.alert);
    }

    #[test]
    fn test_chi_square_numeric_alert() {
        let mut monitor = DistributionMonitor::new(numeric_config(DistributionMetric::ChiSquared));
        monitor.set_reference_numeric(&vec![1.0; 50]);
        let result = monitor.check_numeric(&vec![100.0; 50]).unwrap();
        assert!(result.alert);
    }

    #[test]
    fn test_categorical_drift() {
        let mut monitor = DistributionMonitor::new(numeric_config(DistributionMetric::ChiSquared));
        monitor.set_reference_categorical(&["a".into(), "a".into(), "a".into(), "a".into()]);
        let result = monitor
            .check_categorical(&["b".into(), "b".into(), "b".into(), "b".into()])
            .unwrap();
        assert!(result.alert);
    }

    #[test]
    fn test_categorical_no_drift() {
        let mut monitor = DistributionMonitor::new(numeric_config(DistributionMetric::ChiSquared));
        let labels: Vec<String> = vec!["a".into(), "b".into(), "a".into(), "b".into()];
        monitor.set_reference_categorical(&labels);
        let result = monitor.check_categorical(&labels).unwrap();
        assert!(!result.alert);
    }
}
