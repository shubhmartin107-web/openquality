use crate::error::{OpenQualityError, Result};
use crate::stats::threshold::{ThresholdMethod, auto_threshold};
use crate::types::*;
use chrono::Utc;
use std::collections::HashMap;

pub struct VolumeMonitor {
    pub config: MonitorConfig,
    pub history: Vec<f64>,
}

impl VolumeMonitor {
    pub fn new(config: MonitorConfig) -> Self {
        Self {
            config,
            history: Vec::new(),
        }
    }

    pub fn check(&mut self, current_row_count: u64) -> Result<MonitorResult> {
        if let MonitorType::Volume { .. } = &self.config.monitor_type {
            let count = current_row_count as f64;
            self.history.push(count);
            if self.history.len() > self.config.threshold_config.window_size {
                self.history.remove(0);
            }
            let threshold = if self.config.auto_threshold && self.history.len() > 3 {
                let method = match self.config.threshold_config.method {
                    crate::types::ThresholdMethod::ThreeSigma => ThresholdMethod::ThreeSigma,
                    crate::types::ThresholdMethod::IQR => ThresholdMethod::IQR,
                    crate::types::ThresholdMethod::Mad => ThresholdMethod::Mad,
                    crate::types::ThresholdMethod::Fixed(v) => {
                        return self.fixed_threshold_check(count, v);
                    }
                };
                let at = auto_threshold(
                    &self.history,
                    method,
                    self.config.threshold_config.sensitivity,
                    self.config.threshold_config.min_threshold,
                    self.config.threshold_config.max_threshold,
                )?;
                at.threshold
            } else {
                count * 0.5
            };
            let alert = count > threshold * 1.5 || count < threshold * 0.1;
            let severity = if count == 0.0
                || (alert && (count > threshold * 2.0 || count < threshold * 0.05))
            {
                Severity::Critical
            } else {
                Severity::Warning
            };
            let pct_change = if self.history.len() > 1 {
                let baseline = self.history[..self.history.len() - 1].iter().sum::<f64>()
                    / (self.history.len() - 1) as f64;
                if baseline > 0.0 {
                    (count - baseline) / baseline * 100.0
                } else {
                    0.0
                }
            } else {
                0.0
            };
            let mut details = HashMap::new();
            details.insert("row_count".into(), serde_json::json!(count));
            details.insert("threshold".into(), serde_json::json!(threshold));
            details.insert("pct_change".into(), serde_json::json!(pct_change));
            details.insert("history_len".into(), serde_json::json!(self.history.len()));
            Ok(MonitorResult {
                monitor_id: self.config.id.clone(),
                monitor_type: self.config.monitor_type.clone(),
                table_name: self.config.table_name.clone(),
                alert,
                severity,
                score: count,
                threshold,
                message: if alert {
                    format!(
                        "Volume alert: table '{}' has {} rows (change: {:.1}%)",
                        self.config.table_name, count, pct_change
                    )
                } else {
                    format!(
                        "Volume OK: table '{}' has {} rows",
                        self.config.table_name, count
                    )
                },
                details,
                timestamp: Utc::now(),
            })
        } else {
            Err(OpenQualityError::Monitor("Wrong monitor type".into()))
        }
    }

    fn fixed_threshold_check(&self, count: f64, fixed: f64) -> Result<MonitorResult> {
        let alert = count > fixed * 1.5 || count < fixed * 0.1;
        let mut details = HashMap::new();
        details.insert("row_count".into(), serde_json::json!(count));
        details.insert("threshold".into(), serde_json::json!(fixed));
        Ok(MonitorResult {
            monitor_id: self.config.id.clone(),
            monitor_type: self.config.monitor_type.clone(),
            table_name: self.config.table_name.clone(),
            alert,
            severity: if count == 0.0 {
                Severity::Critical
            } else {
                Severity::Warning
            },
            score: count,
            threshold: fixed,
            message: if alert {
                format!(
                    "Volume alert: table '{}' has {} rows",
                    self.config.table_name, count
                )
            } else {
                format!(
                    "Volume OK: table '{}' has {} rows",
                    self.config.table_name, count
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

    fn config(auto_threshold: bool) -> MonitorConfig {
        let mut c = MonitorConfig::new(
            "volume_test",
            MonitorType::Volume {
                baseline_period_seconds: 3600,
            },
            "test_table",
        );
        c.auto_threshold = auto_threshold;
        c
    }

    #[test]
    fn test_volume_increase_alert() {
        let mut monitor = VolumeMonitor::new(config(false));
        for _ in 0..5 {
            monitor.check(100).unwrap();
        }
        let result = monitor.check(1000).unwrap();
        assert!(result.alert);
        assert!((result.score - 1000.0).abs() < 1e-6);
    }

    #[test]
    fn test_volume_decrease_alert() {
        let mut monitor = VolumeMonitor::new(config(false));
        for _ in 0..5 {
            monitor.check(100).unwrap();
        }
        let result = monitor.check(5).unwrap();
        assert!(result.alert);
    }

    #[test]
    fn test_volume_normal() {
        let mut monitor = VolumeMonitor::new(config(true));
        for _ in 0..5 {
            monitor.check(100).unwrap();
        }
        let result = monitor.check(105).unwrap();
        assert!(!result.alert);
    }

    #[test]
    fn test_fixed_threshold() {
        let mut c = config(false);
        c.threshold_config.method = crate::types::ThresholdMethod::Fixed(200.0);
        c.auto_threshold = true;
        let mut monitor = VolumeMonitor::new(c);
        let result = monitor.check(500).unwrap();
        assert!(result.alert);
    }

    #[test]
    fn test_zero_rows_critical() {
        let mut monitor = VolumeMonitor::new(config(true));
        for _ in 0..5 {
            monitor.check(100).unwrap();
        }
        let result = monitor.check(0).unwrap();
        assert!(result.alert);
        assert_eq!(result.severity, Severity::Critical);
    }
}
