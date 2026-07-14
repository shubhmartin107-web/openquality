use serde_json::Value;
use std::env;

pub struct Client {
    base: String,
    token: Option<String>,
    inner: reqwest::Client,
}

impl Client {
    pub fn from_env() -> Self {
        let base = env::var("OQ_API_URL").unwrap_or_else(|_| "http://localhost:8080".into());
        let token = env::var("OQ_API_TOKEN").ok();
        Self {
            base,
            token,
            inner: reqwest::Client::new(),
        }
    }

    async fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<Value, String> {
        let url = format!("{}{}", self.base, path);
        let mut req = self.inner.request(method, &url);
        if let Some(ref t) = self.token {
            req = req.header("Authorization", format!("Bearer {}", t));
        }
        if let Some(b) = body {
            req = req.header("Content-Type", "application/json").json(&b);
        }
        let res = req
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        let status = res.status();
        let body: Value = res
            .json()
            .await
            .unwrap_or_default();
        if !status.is_success() {
            let err = body
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error");
            return Err(format!("{}: {}", status.as_u16(), err));
        }
        Ok(body)
    }

    pub async fn health(&self) -> Result<Value, String> {
        self.request(reqwest::Method::GET, "/health", None).await
    }

    pub async fn list_workspaces(&self) -> Result<Value, String> {
        self.request(reqwest::Method::GET, "/api/v1/workspaces", None)
            .await
    }

    pub async fn create_workspace(&self, name: &str, slug: &str) -> Result<Value, String> {
        self.request(
            reqwest::Method::POST,
            "/api/v1/workspaces",
            Some(serde_json::json!({"name": name, "slug": slug})),
        )
        .await
    }

    pub async fn list_monitors(&self, ws_id: &str) -> Result<Value, String> {
        self.request(
            reqwest::Method::GET,
            &format!("/api/v1/workspaces/{}/monitors", ws_id),
            None,
        )
        .await
    }

    pub async fn create_monitor(
        &self,
        ws_id: &str,
        name: &str,
        mon_type: &str,
        table: &str,
        cron: Option<&str>,
    ) -> Result<Value, String> {
        let mut body =
            serde_json::json!({"name": name, "monitor_type": mon_type, "table_name": table});
        if let Some(c) = cron {
            body["schedule_cron"] = serde_json::json!(c);
        }
        self.request(
            reqwest::Method::POST,
            &format!("/api/v1/workspaces/{}/monitors", ws_id),
            Some(body),
        )
        .await
    }

    pub async fn delete_monitor(&self, id: &str) -> Result<Value, String> {
        self.request(
            reqwest::Method::DELETE,
            &format!("/api/v1/monitors/{}", id),
            None,
        )
        .await
    }

    pub async fn run_monitor(&self, id: &str) -> Result<Value, String> {
        self.request(
            reqwest::Method::POST,
            &format!("/api/v1/monitors/{}/run", id),
            None,
        )
        .await
    }

    pub async fn list_incidents(&self) -> Result<Value, String> {
        self.request(reqwest::Method::GET, "/api/v1/incidents", None)
            .await
    }

    pub async fn get_incident(&self, id: &str) -> Result<Value, String> {
        self.request(
            reqwest::Method::GET,
            &format!("/api/v1/incidents/{}", id),
            None,
        )
        .await
    }

    pub async fn acknowledge_incident(&self, id: &str) -> Result<Value, String> {
        self.request(
            reqwest::Method::POST,
            &format!("/api/v1/incidents/{}/acknowledge", id),
            None,
        )
        .await
    }

    pub async fn resolve_incident(&self, id: &str) -> Result<Value, String> {
        self.request(
            reqwest::Method::POST,
            &format!("/api/v1/incidents/{}/resolve", id),
            None,
        )
        .await
    }

    pub async fn list_data_sources(&self, ws_id: &str) -> Result<Value, String> {
        self.request(
            reqwest::Method::GET,
            &format!("/api/v1/workspaces/{}/data-sources", ws_id),
            None,
        )
        .await
    }

    pub async fn create_data_source(
        &self,
        ws_id: &str,
        name: &str,
        conn_type: &str,
        conn_str: &str,
    ) -> Result<Value, String> {
        self.request(reqwest::Method::POST, &format!("/api/v1/workspaces/{}/data-sources", ws_id),
            Some(serde_json::json!({"name": name, "connector_type": conn_type, "config": {"connection_string": conn_str}}))).await
    }

    pub async fn delete_data_source(&self, id: &str) -> Result<Value, String> {
        self.request(
            reqwest::Method::DELETE,
            &format!("/api/v1/data-sources/{}", id),
            None,
        )
        .await
    }

    pub async fn dbt_parse_manifest(&self, manifest: Value) -> Result<Value, String> {
        self.request(
            reqwest::Method::POST,
            "/api/v1/integrations/dbt/parse-manifest",
            Some(serde_json::json!({"manifest_json": manifest})),
        )
        .await
    }

    pub async fn ge_translate(&self, suite: Value) -> Result<Value, String> {
        self.request(
            reqwest::Method::POST,
            "/api/v1/integrations/ge/translate",
            Some(serde_json::json!({"suite_json": suite})),
        )
        .await
    }

    pub async fn lineage_parse_sql(&self, sql: &str) -> Result<Value, String> {
        self.request(
            reqwest::Method::POST,
            "/api/v1/integrations/lineage/parse-sql",
            Some(serde_json::json!({"sql": sql})),
        )
        .await
    }

    pub async fn lineage_build_graph(&self, statements: &[String]) -> Result<Value, String> {
        self.request(
            reqwest::Method::POST,
            "/api/v1/integrations/lineage/build-graph",
            Some(serde_json::json!({"statements": statements})),
        )
        .await
    }
}
