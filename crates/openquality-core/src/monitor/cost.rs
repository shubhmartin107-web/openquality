use crate::error::Result;
use crate::types::*;
use chrono::Utc;
use std::collections::HashMap;

pub struct CostMonitor;

impl CostMonitor {
    pub fn check(
        current_cost: f64,
        resource_type: &CostResourceType,
        budget: f64,
    ) -> Result<MonitorResult> {
        let alert = current_cost > budget;
        let ratio = if budget > 0.0 {
            current_cost / budget
        } else {
            1.0
        };

        let mut details = HashMap::new();
        details.insert("current_cost".into(), serde_json::json!(current_cost));
        details.insert("budget".into(), serde_json::json!(budget));
        details.insert("ratio".into(), serde_json::json!(ratio));
        details.insert(
            "resource_type".into(),
            serde_json::json!(format!("{:?}", resource_type)),
        );

        Ok(MonitorResult {
            monitor_id: format!("cost_{:?}", resource_type),
            monitor_type: MonitorType::Cost {
                resource_type: resource_type.clone(),
                budget,
            },
            table_name: String::new(),
            alert,
            severity: if alert && ratio > 2.0 {
                Severity::Critical
            } else if alert {
                Severity::Warning
            } else {
                Severity::Info
            },
            score: current_cost,
            threshold: budget,
            message: if alert {
                format!(
                    "Cost alert: {:?} cost={:.2} exceeds budget={:.2} (x{:.1})",
                    resource_type, current_cost, budget, ratio
                )
            } else {
                format!(
                    "Cost OK: {:?} cost={:.2} within budget={:.2}",
                    resource_type, current_cost, budget
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
    fn test_under_budget() {
        let result = CostMonitor::check(50.0, &CostResourceType::QueryCost, 100.0).unwrap();
        assert!(!result.alert);
    }

    #[test]
    fn test_over_budget() {
        let result = CostMonitor::check(150.0, &CostResourceType::StorageCost, 100.0).unwrap();
        assert!(result.alert);
        assert_eq!(result.severity, Severity::Warning);
    }

    #[test]
    fn test_critical_overage() {
        let result = CostMonitor::check(300.0, &CostResourceType::ComputeCost, 100.0).unwrap();
        assert!(result.alert);
        assert_eq!(result.severity, Severity::Critical);
    }
}
