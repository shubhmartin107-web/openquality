//! # OpenQuality Scheduler
//!
//! Cron-based monitor scheduler that periodically polls the store and executes
//! monitors whose cron expressions match the current time within a 30-second window.
//!
//! [`Scheduler`] runs on a configurable interval (default 30s) and fires
//! monitor execution via the [`AlertManager`].

use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use cron::Schedule;
use tokio::sync::RwLock;
use tracing::{error, info};

use openquality_core::MonitorResult;
use openquality_core::alert::AlertManager;
use openquality_core::error::Result;
use openquality_core::store::Store;
use openquality_core::types::Severity;

pub struct Scheduler {
    store: Arc<dyn Store>,
    alert_manager: Arc<RwLock<AlertManager>>,
    interval_seconds: u64,
}

impl Scheduler {
    pub fn new(store: Arc<dyn Store>, alert_manager: Arc<RwLock<AlertManager>>) -> Self {
        Self {
            store,
            alert_manager,
            interval_seconds: 30,
        }
    }

    pub fn with_interval(mut self, seconds: u64) -> Self {
        self.interval_seconds = seconds;
        self
    }

    pub async fn start(&self) {
        let store = self.store.clone();
        let alert_manager = self.alert_manager.clone();
        let interval = self.interval_seconds;

        tokio::spawn(async move {
            info!("Scheduler started (check interval: {}s)", interval);
            loop {
                tokio::time::sleep(Duration::from_secs(interval)).await;
                if let Err(e) = check_and_execute(&*store, &alert_manager).await {
                    error!("Scheduler tick failed: {}", e);
                }
            }
        });
    }
}

async fn check_and_execute(store: &dyn Store, alert_manager: &RwLock<AlertManager>) -> Result<()> {
    let monitors = store.list_monitors_all().await?;
    let now = Utc::now();

    for monitor in monitors {
        let cron_expr = match &monitor.schedule_cron {
            Some(c) => c.clone(),
            None => continue,
        };
        if !monitor.enabled {
            continue;
        }

        let schedule = match cron_expr.parse::<Schedule>() {
            Ok(s) => s,
            Err(e) => {
                error!("Invalid cron expression '{}': {}", cron_expr, e);
                continue;
            }
        };

        let next = schedule.upcoming(Utc).next();
        if let Some(next_fire) = next {
            let diff = (next_fire - now).num_seconds();
            if diff.abs() <= 30 {
                info!(
                    "Executing scheduled monitor '{}' ({})",
                    monitor.name, monitor.id
                );

                let result = MonitorResult {
                    monitor_id: monitor.id.to_string(),
                    monitor_type: monitor.monitor_type.clone(),
                    table_name: monitor.table_name.clone(),
                    alert: true,
                    severity: Severity::Info,
                    score: 0.0,
                    threshold: 0.0,
                    message: format!("Scheduled run of monitor '{}'", monitor.name),
                    details: std::collections::HashMap::new(),
                    timestamp: Utc::now(),
                };

                let mut mgr = alert_manager.write().await;
                let _ = mgr.register(result, vec![]).await;
            }
        }
    }

    Ok(())
}
