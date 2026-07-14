use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

use crate::alert::channel::AlertChannel;
use crate::error::Result;
use crate::store::{InMemoryStore, IncidentUpdate, Store};
use crate::types::*;

pub struct AlertManager {
    store: Arc<dyn Store>,
    channels: Vec<Box<dyn AlertChannel + Send + Sync>>,
    dedup_window_seconds: i64,
    recent_incidents: Vec<(String, DateTime<Utc>)>,
    ignore_rules: Vec<IgnoreRule>,
    maintenance_windows: Vec<MaintenanceWindow>,
}

impl AlertManager {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self {
            store,
            channels: Vec::new(),
            dedup_window_seconds: 300,
            recent_incidents: Vec::new(),
            ignore_rules: Vec::new(),
            maintenance_windows: Vec::new(),
        }
    }

    pub fn add_channel(&mut self, channel: Box<dyn AlertChannel + Send + Sync>) {
        self.channels.push(channel);
    }

    pub fn set_dedup_window(&mut self, seconds: i64) {
        self.dedup_window_seconds = seconds;
    }

    pub fn add_ignore_rule(&mut self, rule: IgnoreRule) {
        self.ignore_rules.push(rule);
    }

    pub fn add_maintenance_window(&mut self, window: MaintenanceWindow) {
        self.maintenance_windows.push(window);
    }

    fn is_in_maintenance(&self, monitor_id: &str) -> bool {
        let now = Utc::now();
        self.maintenance_windows.iter().any(|w| {
            let matches_monitor = w.monitor_id.as_ref().is_none_or(|m| m == monitor_id);
            matches_monitor && now >= w.starts_at && now <= w.ends_at
        })
    }

    fn is_ignored(&self, incident: &Incident) -> bool {
        self.ignore_rules.iter().any(|rule| {
            if let Some(expires) = rule.expires_at {
                if Utc::now() > expires {
                    return false;
                }
            }
            if let Some(ref mid) = rule.monitor_id {
                if mid != &incident.monitor_id {
                    return false;
                }
            }
            if let Some(ref pattern) = rule.pattern {
                if !incident.message.contains(pattern) {
                    return false;
                }
            }
            true
        })
    }

    fn is_duplicate(&mut self, group_key: &str) -> bool {
        let cutoff = Utc::now() - chrono::Duration::seconds(self.dedup_window_seconds);
        self.recent_incidents.retain(|(_, t)| *t > cutoff);
        let is_dup = self.recent_incidents.iter().any(|(k, _)| k == group_key);
        if !is_dup {
            self.recent_incidents
                .push((group_key.to_string(), Utc::now()));
        }
        is_dup
    }

    pub async fn register(
        &mut self,
        result: MonitorResult,
        hints: Vec<String>,
    ) -> Result<Option<Incident>> {
        let mut incident = Incident::new(result, hints);

        if self.is_in_maintenance(&incident.monitor_id) {
            incident.severity = Severity::Info;
            incident.message = format!("[SUPPRESSED] {}", incident.message);
        }

        if self.is_ignored(&incident) {
            incident.severity = Severity::Info;
            incident.message = format!("[IGNORED] {}", incident.message);
        }

        if let Some(ref group_key) = incident.group_key {
            if self.is_duplicate(group_key) {
                incident.severity = Severity::Info;
                incident.message = format!("[DEDUP] {}", incident.message);
            }
        }

        self.store.create_incident(&incident).await?;

        if incident.severity != Severity::Info {
            for channel in &self.channels {
                let _ = channel.send(&incident).await;
            }
        }

        Ok(Some(incident))
    }

    pub async fn resolve(&self, incident_id: Uuid) -> Result<Option<Incident>> {
        let updates = IncidentUpdate {
            resolved: Some(true),
            resolved_at: Some(Utc::now()),
            ..Default::default()
        };
        self.store.update_incident(incident_id, updates).await?;
        self.store.get_incident(incident_id).await
    }

    pub async fn acknowledge(
        &self,
        incident_id: Uuid,
        by: Option<Uuid>,
    ) -> Result<Option<Incident>> {
        let updates = IncidentUpdate {
            acked: Some(true),
            acknowledged_at: Some(Utc::now()),
            owner_id: by,
            ..Default::default()
        };
        self.store.update_incident(incident_id, updates).await?;
        self.store.get_incident(incident_id).await
    }

    pub async fn snooze(
        &self,
        incident_id: Uuid,
        until: DateTime<Utc>,
    ) -> Result<Option<Incident>> {
        let updates = IncidentUpdate {
            snoozed_until: Some(until),
            ..Default::default()
        };
        self.store.update_incident(incident_id, updates).await?;
        self.store.get_incident(incident_id).await
    }

    pub async fn list(&self, status: Option<&str>) -> Result<Vec<Incident>> {
        self.store.list_incidents(Uuid::nil(), status).await
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Incident>> {
        self.store.get_incident(id).await
    }

    pub async fn get_by_monitor(&self, monitor_id: &str) -> Result<Vec<Incident>> {
        let all = self.store.list_incidents(Uuid::nil(), None).await?;
        Ok(all
            .into_iter()
            .filter(|i| i.monitor_id == monitor_id)
            .collect())
    }

    pub async fn open_count(&self) -> Result<usize> {
        let incidents = self.store.list_incidents(Uuid::nil(), Some("open")).await?;
        Ok(incidents.len())
    }

    pub fn channels(&self) -> &[Box<dyn AlertChannel + Send + Sync>] {
        &self.channels
    }

    pub fn channels_mut(&mut self) -> &mut Vec<Box<dyn AlertChannel + Send + Sync>> {
        &mut self.channels
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self {
            store: Arc::new(InMemoryStore::new()),
            channels: Vec::new(),
            dedup_window_seconds: 300,
            recent_incidents: Vec::new(),
            ignore_rules: Vec::new(),
            maintenance_windows: Vec::new(),
        }
    }
}
