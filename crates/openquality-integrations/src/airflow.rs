use crate::IntegrationError;
use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AirflowWebhookPayload {
    #[serde(rename = "dag_id")]
    pub dag_id: String,
    #[serde(rename = "run_id")]
    pub run_id: Option<String>,
    pub state: Option<String>,
    #[serde(rename = "execution_date")]
    pub execution_date: Option<String>,
    #[serde(rename = "start_date")]
    pub start_date: Option<String>,
    #[serde(rename = "end_date")]
    pub end_date: Option<String>,
    #[serde(rename = "task_id")]
    pub task_id: Option<String>,
    pub duration: Option<f64>,
    pub operator: Option<String>,
    #[serde(rename = "dag_run_url")]
    pub dag_run_url: Option<String>,
    pub event: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum AirflowEvent {
    DagRunQueued,
    DagRunRunning,
    DagRunSuccess,
    DagRunFailed,
    TaskInstanceRunning,
    TaskInstanceSuccess,
    TaskInstanceFailed,
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct ParsedAirflowEvent {
    pub event: AirflowEvent,
    pub dag_id: String,
    pub run_id: String,
    pub execution_date: Option<DateTime<Utc>>,
    pub duration_seconds: Option<f64>,
    pub task_id: Option<String>,
    pub dag_run_url: Option<String>,
}

impl AirflowWebhookPayload {
    pub fn parse(&self) -> Result<ParsedAirflowEvent, IntegrationError> {
        let event = match (self.event.as_deref(), self.state.as_deref()) {
            (_, Some("queued")) | (Some("dag_run_queued"), _) => AirflowEvent::DagRunQueued,
            (_, Some("running"))
            | (Some("dag_run_running"), _)
            | (Some("running"), _)
            | (Some("started"), _) => AirflowEvent::DagRunRunning,
            (_, Some("success")) | (Some("dag_run_success"), _) | (Some("success"), _) => {
                AirflowEvent::DagRunSuccess
            }
            (_, Some("failed")) | (Some("dag_run_failed"), _) | (Some("failed"), _) => {
                AirflowEvent::DagRunFailed
            }
            (Some("task_instance_running"), _) => AirflowEvent::TaskInstanceRunning,
            (Some("task_instance_success"), _) => AirflowEvent::TaskInstanceSuccess,
            (Some("task_instance_failed"), _) => AirflowEvent::TaskInstanceFailed,
            (Some(other), _) => AirflowEvent::Unknown(other.to_string()),
            (None, Some(s)) => AirflowEvent::Unknown(s.to_string()),
            (None, None) => AirflowEvent::Unknown("no_event".into()),
        };

        let exec_date = self
            .execution_date
            .as_deref()
            .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
            .map(|d| d.with_timezone(&Utc));

        Ok(ParsedAirflowEvent {
            event,
            dag_id: self.dag_id.clone(),
            run_id: self.run_id.clone().unwrap_or_default(),
            execution_date: exec_date,
            duration_seconds: self.duration,
            task_id: self.task_id.clone(),
            dag_run_url: self.dag_run_url.clone(),
        })
    }
}

impl ParsedAirflowEvent {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.event,
            AirflowEvent::DagRunSuccess
                | AirflowEvent::DagRunFailed
                | AirflowEvent::TaskInstanceSuccess
                | AirflowEvent::TaskInstanceFailed
        )
    }

    pub fn is_success(&self) -> bool {
        matches!(
            self.event,
            AirflowEvent::DagRunSuccess | AirflowEvent::TaskInstanceSuccess
        )
    }

    pub fn is_failure(&self) -> bool {
        matches!(
            self.event,
            AirflowEvent::DagRunFailed | AirflowEvent::TaskInstanceFailed
        )
    }

    pub fn affected_tables(&self) -> Vec<String> {
        let dag = self.dag_id.to_lowercase();
        let prefixes = [
            "load_",
            "ingest_",
            "etl_",
            "sync_",
            "refresh_",
            "transform_",
        ];
        for p in &prefixes {
            if let Some(rest) = dag.strip_prefix(p) {
                return vec![rest.replace('_', ".")];
            }
        }
        vec![dag.replace('_', ".")]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dag_success() {
        let payload = AirflowWebhookPayload {
            dag_id: "load_orders".into(),
            run_id: Some("manual_001".into()),
            state: Some("success".into()),
            execution_date: Some("2026-07-12T10:00:00Z".into()),
            start_date: None,
            end_date: None,
            task_id: None,
            duration: Some(120.5),
            operator: None,
            dag_run_url: Some("http://airflow/dags/load_orders".into()),
            event: Some("dag_run_success".into()),
        };
        let parsed = payload.parse().unwrap();
        assert!(matches!(parsed.event, AirflowEvent::DagRunSuccess));
        assert_eq!(parsed.dag_id, "load_orders");
        assert_eq!(parsed.run_id, "manual_001");
        assert!(parsed.is_terminal());
        assert!(parsed.is_success());
    }

    #[test]
    fn test_parse_dag_failed() {
        let payload = AirflowWebhookPayload {
            dag_id: "etl_daily".into(),
            run_id: Some("sched_20260712".into()),
            state: Some("failed".into()),
            execution_date: None,
            start_date: None,
            end_date: None,
            task_id: None,
            duration: None,
            operator: None,
            dag_run_url: None,
            event: Some("dag_run_failed".into()),
        };
        let parsed = payload.parse().unwrap();
        assert!(matches!(parsed.event, AirflowEvent::DagRunFailed));
        assert!(parsed.is_terminal());
        assert!(parsed.is_failure());
    }

    #[test]
    fn test_affected_tables() {
        let payload = AirflowWebhookPayload {
            dag_id: "load_orders".into(),
            run_id: None,
            state: Some("success".into()),
            execution_date: None,
            start_date: None,
            end_date: None,
            task_id: None,
            duration: None,
            operator: None,
            dag_run_url: None,
            event: None,
        };
        assert_eq!(payload.parse().unwrap().affected_tables(), vec!["orders"]);
    }

    #[test]
    fn test_dag_running() {
        let payload = AirflowWebhookPayload {
            dag_id: "sync_users".into(),
            run_id: Some("run_1".into()),
            state: Some("running".into()),
            execution_date: None,
            start_date: None,
            end_date: None,
            task_id: None,
            duration: None,
            operator: None,
            dag_run_url: None,
            event: None,
        };
        let parsed = payload.parse().unwrap();
        assert!(matches!(parsed.event, AirflowEvent::DagRunRunning));
        assert!(!parsed.is_terminal());
    }
}
