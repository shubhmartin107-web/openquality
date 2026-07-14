use async_trait::async_trait;
use reqwest::Client;

use crate::error::Result;
use crate::types::Incident;

#[async_trait]
pub trait AlertChannel: Send + Sync {
    async fn send(&self, incident: &Incident) -> Result<()>;
    fn name(&self) -> &str;
}

pub struct StdoutAlertChannel {
    pub name: String,
}

impl StdoutAlertChannel {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[async_trait]
impl AlertChannel for StdoutAlertChannel {
    async fn send(&self, incident: &Incident) -> Result<()> {
        let severity = &incident.severity;
        let ts = incident.timestamp.format("%Y-%m-%dT%H:%M:%S%.3fZ");
        println!(
            "[{}] [{}] {} — monitor={} hints={:?}",
            ts, severity, incident.message, incident.monitor_id, incident.root_cause_hints
        );
        if !incident.root_cause_hints.is_empty() {
            for hint in &incident.root_cause_hints {
                println!("  └─ {}", hint);
            }
        }
        Ok(())
    }
    fn name(&self) -> &str {
        &self.name
    }
}

pub struct JsonAlertChannel {
    pub name: String,
}

impl JsonAlertChannel {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[async_trait]
impl AlertChannel for JsonAlertChannel {
    async fn send(&self, incident: &Incident) -> Result<()> {
        let json = serde_json::to_string_pretty(incident)?;
        println!("{}", json);
        Ok(())
    }
    fn name(&self) -> &str {
        &self.name
    }
}

pub struct SlackAlertChannel {
    pub name: String,
    pub webhook_url: String,
    client: Client,
}

impl SlackAlertChannel {
    pub fn new(name: &str, webhook_url: &str) -> Self {
        Self {
            name: name.to_string(),
            webhook_url: webhook_url.to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl AlertChannel for SlackAlertChannel {
    async fn send(&self, incident: &Incident) -> Result<()> {
        let text = format!(
            "[{}] {} — monitor={}",
            incident.severity, incident.message, incident.monitor_id
        );
        let payload = serde_json::json!({ "text": text });
        let _ = self
            .client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await;
        tracing::info!("Slack channel '{}' sent alert", self.name);
        Ok(())
    }
    fn name(&self) -> &str {
        &self.name
    }
}

pub struct PagerDutyAlertChannel {
    pub name: String,
    pub routing_key: String,
    client: Client,
}

impl PagerDutyAlertChannel {
    pub fn new(name: &str, routing_key: &str) -> Self {
        Self {
            name: name.to_string(),
            routing_key: routing_key.to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl AlertChannel for PagerDutyAlertChannel {
    async fn send(&self, incident: &Incident) -> Result<()> {
        let payload = serde_json::json!({
            "routing_key": self.routing_key,
            "event_action": "trigger",
            "payload": {
                "summary": incident.message,
                "severity": "critical",
                "source": incident.monitor_id,
            }
        });
        let _ = self
            .client
            .post("https://events.pagerduty.com/v2/enqueue")
            .json(&payload)
            .send()
            .await;
        tracing::info!("PagerDuty channel '{}' sent alert", self.name);
        Ok(())
    }
    fn name(&self) -> &str {
        &self.name
    }
}

pub struct WebhookAlertChannel {
    pub name: String,
    pub url: String,
    client: Client,
}

impl WebhookAlertChannel {
    pub fn new(name: &str, url: &str) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl AlertChannel for WebhookAlertChannel {
    async fn send(&self, incident: &Incident) -> Result<()> {
        let _ = self.client.post(&self.url).json(incident).send().await;
        tracing::info!("Webhook channel '{}' sent alert to {}", self.name, self.url);
        Ok(())
    }
    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use std::collections::HashMap;

    fn sample_incident() -> Incident {
        Incident {
            id: uuid::Uuid::new_v4(),
            workspace_id: Some(uuid::Uuid::new_v4()),
            monitor_id: "test_monitor".into(),
            monitor_result: MonitorResult {
                monitor_id: "test_monitor".into(),
                monitor_type: MonitorType::Schema,
                table_name: "test_table".into(),
                alert: true,
                severity: Severity::Warning,
                score: 1.0,
                threshold: 0.5,
                message: "Test alert".into(),
                details: HashMap::new(),
                timestamp: chrono::Utc::now(),
            },
            severity: Severity::Warning,
            message: "Test alert message".into(),
            detail: serde_json::Value::Null,
            root_cause_hints: vec!["Possible upstream failure".into()],
            timestamp: chrono::Utc::now(),
            resolved: false,
            resolved_at: None,
            acked: false,
            acknowledged_at: None,
            group_key: None,
            escalation_level: 0,
            owner_id: None,
            snoozed_until: None,
        }
    }

    #[tokio::test]
    async fn test_stdout_channel_name() {
        let ch = StdoutAlertChannel::new("stdout-test");
        assert_eq!(ch.name(), "stdout-test");
    }

    #[tokio::test]
    async fn test_stdout_channel_send() {
        let ch = StdoutAlertChannel::new("stdout-test");
        let result = ch.send(&sample_incident()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_json_channel_name() {
        let ch = JsonAlertChannel::new("json-test");
        assert_eq!(ch.name(), "json-test");
    }

    #[tokio::test]
    async fn test_json_channel_send() {
        let ch = JsonAlertChannel::new("json-test");
        let result = ch.send(&sample_incident()).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_slack_channel_construction() {
        let ch = SlackAlertChannel::new("slack-test", "https://hooks.example.com");
        assert_eq!(ch.name(), "slack-test");
        assert_eq!(ch.webhook_url, "https://hooks.example.com");
    }

    #[test]
    fn test_pagerduty_channel_construction() {
        let ch = PagerDutyAlertChannel::new("pd-test", "routing_key_123");
        assert_eq!(ch.name(), "pd-test");
        assert_eq!(ch.routing_key, "routing_key_123");
    }

    #[test]
    fn test_webhook_channel_construction() {
        let ch = WebhookAlertChannel::new("webhook-test", "https://endpoint.example.com/hook");
        assert_eq!(ch.name(), "webhook-test");
        assert_eq!(ch.url, "https://endpoint.example.com/hook");
    }
}
