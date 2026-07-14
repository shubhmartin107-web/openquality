use crate::error::{OpenQualityError, Result};
use crate::types::*;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub struct FreshnessMonitor {
    pub config: MonitorConfig,
    pub last_updated: Option<DateTime<Utc>>,
    pub history: Vec<f64>,
}

impl FreshnessMonitor {
    pub fn new(config: MonitorConfig) -> Self {
        Self {
            config,
            last_updated: None,
            history: Vec::new(),
        }
    }

    pub fn check(
        &self,
        current_time: DateTime<Utc>,
        last_data_time: DateTime<Utc>,
    ) -> Result<MonitorResult> {
        if let MonitorType::Freshness { max_age_seconds } = &self.config.monitor_type {
            let age_secs = (current_time - last_data_time).num_seconds() as f64;
            let threshold = if self.config.auto_threshold && !self.history.is_empty() {
                let mut h = self.history.clone();
                h.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let n = h.len();
                let med = if n % 2 == 0 {
                    (h[n / 2 - 1] + h[n / 2]) / 2.0
                } else {
                    h[n / 2]
                };
                let deviations: Vec<f64> = h.iter().map(|v| (v - med).abs()).collect();
                let mut ds = deviations.clone();
                ds.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let mad = if ds.len() % 2 == 0 {
                    (ds[ds.len() / 2 - 1] + ds[ds.len() / 2]) / 2.0
                } else {
                    ds[ds.len() / 2]
                };
                let dynamic = med + self.config.threshold_config.sensitivity * 1.4826 * mad;
                dynamic.max(*max_age_seconds as f64)
            } else {
                *max_age_seconds as f64
            };
            let alert = age_secs > threshold;
            let severity = if alert && age_secs > threshold * 2.0 {
                Severity::Critical
            } else {
                Severity::Warning
            };
            let mut details = HashMap::new();
            details.insert("age_seconds".into(), serde_json::json!(age_secs));
            details.insert("threshold".into(), serde_json::json!(threshold));
            details.insert(
                "last_data_time".into(),
                serde_json::json!(last_data_time.to_rfc3339()),
            );
            Ok(MonitorResult {
                monitor_id: self.config.id.clone(),
                monitor_type: self.config.monitor_type.clone(),
                table_name: self.config.table_name.clone(),
                alert,
                severity,
                score: age_secs,
                threshold,
                message: if alert {
                    format!(
                        "Freshness alert: table '{}' last updated {}s ago (threshold: {}s)",
                        self.config.table_name, age_secs as u64, threshold as u64
                    )
                } else {
                    format!(
                        "Freshness OK: table '{}' updated {}s ago",
                        self.config.table_name, age_secs as u64
                    )
                },
                details,
                timestamp: Utc::now(),
            })
        } else {
            Err(OpenQualityError::Monitor("Wrong monitor type".into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn config(max_age_seconds: u64, auto_threshold: bool) -> MonitorConfig {
        let mut c = MonitorConfig::new(
            "freshness_test",
            MonitorType::Freshness { max_age_seconds },
            "test_table",
        );
        c.auto_threshold = auto_threshold;
        c
    }

    #[test]
    fn test_age_exceeds_threshold() {
        let monitor = FreshnessMonitor::new(config(3600, false));
        let now = Utc::now();
        let past = now - Duration::seconds(7200);
        let result = monitor.check(now, past).unwrap();
        assert!(result.alert);
    }

    #[test]
    fn test_age_within_threshold() {
        let monitor = FreshnessMonitor::new(config(3600, false));
        let now = Utc::now();
        let past = now - Duration::seconds(1800);
        let result = monitor.check(now, past).unwrap();
        assert!(!result.alert);
    }

    #[test]
    fn test_auto_threshold() {
        let mut c = config(3600, true);
        c.threshold_config.sensitivity = 1.0;
        let mut monitor = FreshnessMonitor::new(c);
        monitor.history = vec![100.0, 110.0, 105.0, 115.0, 108.0];
        let now = Utc::now();
        let past = now - Duration::seconds(200);
        let result = monitor.check(now, past).unwrap();
        assert!(result.score > 0.0);
    }

    #[test]
    fn test_critical_severity() {
        let monitor = FreshnessMonitor::new(config(3600, false));
        let now = Utc::now();
        let past = now - Duration::seconds(86400);
        let result = monitor.check(now, past).unwrap();
        assert!(result.alert);
        assert_eq!(result.severity, Severity::Critical);
    }

    #[test]
    fn test_wrong_monitor_type() {
        let c = MonitorConfig::new("bad", MonitorType::Schema, "t");
        let monitor = FreshnessMonitor::new(c);
        let result = monitor.check(Utc::now(), Utc::now());
        assert!(result.is_err());
    }
}
