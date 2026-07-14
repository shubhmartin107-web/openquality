use crate::IntegrationError;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct DbtManifest {
    pub metadata: Option<DbtMetadata>,
    pub nodes: HashMap<String, DbtNode>,
    pub sources: HashMap<String, DbtSource>,
    pub child_map: Option<HashMap<String, Vec<String>>>,
    pub parent_map: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DbtMetadata {
    pub dbt_schema_version: Option<String>,
    pub dbt_version: Option<String>,
    pub generated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DbtNode {
    pub name: String,
    #[serde(rename = "resource_type")]
    pub resource_type: String,
    pub database: Option<String>,
    pub schema: Option<String>,
    pub alias: Option<String>,
    #[serde(rename = "relation_name")]
    pub relation_name: Option<String>,
    pub config: Option<Value>,
    pub columns: Option<HashMap<String, DbtColumn>>,
    pub tests: Option<Vec<DbtTest>>,
    pub depends_on: Option<DbtDependsOn>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DbtSource {
    pub name: String,
    pub database: Option<String>,
    pub schema: Option<String>,
    pub relation_name: Option<String>,
    pub source_name: Option<String>,
    pub columns: Option<HashMap<String, DbtColumn>>,
    pub freshness: Option<DbtFreshness>,
    pub loaded_at_field: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DbtColumn {
    pub name: Option<String>,
    pub data_type: Option<String>,
    pub description: Option<String>,
    pub meta: Option<Value>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DbtTest {
    pub name: Option<String>,
    pub test_metadata: Option<DbtTestMetadata>,
    pub severity: Option<String>,
    pub column_name: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DbtTestMetadata {
    pub name: Option<String>,
    pub kwargs: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DbtDependsOn {
    pub nodes: Option<Vec<String>>,
    pub macros: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DbtFreshness {
    pub warn_after: Option<DbtFreshnessThreshold>,
    pub error_after: Option<DbtFreshnessThreshold>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DbtFreshnessThreshold {
    pub count: Option<i64>,
    pub period: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DbtModelSummary {
    pub name: String,
    pub database: String,
    pub schema: String,
    pub alias: String,
    pub columns: Vec<String>,
    pub upstream_models: Vec<String>,
    pub downstream_models: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DbtSourceSummary {
    pub name: String,
    pub source_name: String,
    pub database: String,
    pub schema: String,
    pub columns: Vec<String>,
    pub freshness_warn: Option<String>,
}

impl DbtManifest {
    pub fn from_json(json: &str) -> Result<Self, IntegrationError> {
        serde_json::from_str(json).map_err(|e| IntegrationError::Parse(e.to_string()))
    }

    pub fn list_models(&self) -> Vec<DbtModelSummary> {
        let child_map = self.child_map.as_ref();
        let parent_map = self.parent_map.as_ref();

        self.nodes
            .iter()
            .filter(|(_, n)| n.resource_type == "model")
            .map(|(key, node)| {
                let upstream = parent_map
                    .and_then(|m| m.get(key))
                    .map(|v| {
                        v.iter()
                            .filter(|p| self.nodes.contains_key(*p))
                            .map(|p| self.nodes[p].name.clone())
                            .collect()
                    })
                    .unwrap_or_default();

                let downstream = child_map
                    .and_then(|m| m.get(key))
                    .map(|v| {
                        v.iter()
                            .filter(|c| self.nodes.contains_key(*c))
                            .map(|c| self.nodes[c].name.clone())
                            .collect()
                    })
                    .unwrap_or_default();

                DbtModelSummary {
                    name: node.name.clone(),
                    database: node.database.clone().unwrap_or_default(),
                    schema: node.schema.clone().unwrap_or_default(),
                    alias: node.alias.clone().unwrap_or_default(),
                    columns: node
                        .columns
                        .as_ref()
                        .map(|c| c.keys().cloned().collect())
                        .unwrap_or_default(),
                    upstream_models: upstream,
                    downstream_models: downstream,
                    tags: node.tags.clone().unwrap_or_default(),
                }
            })
            .collect()
    }

    pub fn list_sources(&self) -> Vec<DbtSourceSummary> {
        self.sources
            .values()
            .map(|s| {
                let freshness_warn = s.freshness.as_ref().and_then(|f| {
                    f.warn_after.as_ref().map(|w| {
                        format!(
                            "{} {}",
                            w.count.unwrap_or(0),
                            w.period.as_deref().unwrap_or("hours")
                        )
                    })
                });

                DbtSourceSummary {
                    name: s.name.clone(),
                    source_name: s.source_name.clone().unwrap_or_default(),
                    database: s.database.clone().unwrap_or_default(),
                    schema: s.schema.clone().unwrap_or_default(),
                    columns: s
                        .columns
                        .as_ref()
                        .map(|c| c.keys().cloned().collect())
                        .unwrap_or_default(),
                    freshness_warn,
                }
            })
            .collect()
    }

    pub fn extract_lineage_edges(&self) -> Vec<(String, String, String)> {
        let mut edges = Vec::new();
        for node in self.nodes.values() {
            if node.resource_type != "model" {
                continue;
            }
            if let Some(deps) = &node.depends_on {
                if let Some(parents) = &deps.nodes {
                    for parent in parents {
                        let parent_name = self
                            .nodes
                            .get(parent)
                            .map(|n| n.name.clone())
                            .unwrap_or_else(|| parent.clone());
                        edges.push((parent_name, node.name.clone(), "dbt_model".into()));
                    }
                }
            }
        }
        edges
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_manifest() {
        let json = r#"{"metadata":{},"nodes":{},"sources":{}}"#;
        let manifest = DbtManifest::from_json(json).unwrap();
        assert!(manifest.list_models().is_empty());
        assert!(manifest.list_sources().is_empty());
    }

    #[test]
    fn test_parse_model() {
        let json = r#"{
            "metadata": {"dbt_version": "1.8"},
            "nodes": {
                "model.my_project.my_model": {
                    "name": "my_model",
                    "resource_type": "model",
                    "database": "analytics",
                    "schema": "public",
                    "alias": "my_model",
                    "columns": {
                        "id": {"name": "id", "data_type": "integer"},
                        "name": {"name": "name", "data_type": "text"}
                    },
                    "depends_on": {"nodes": ["model.my_project.upstream"], "macros": []},
                    "tags": ["hourly"]
                },
                "model.my_project.upstream": {
                    "name": "upstream",
                    "resource_type": "model",
                    "database": "analytics",
                    "schema": "staging",
                    "alias": "upstream",
                    "columns": {},
                    "tags": []
                }
            },
            "sources": {},
            "child_map": {
                "model.my_project.upstream": ["model.my_project.my_model"]
            },
            "parent_map": {
                "model.my_project.my_model": ["model.my_project.upstream"]
            }
        }"#;
        let manifest = DbtManifest::from_json(json).unwrap();
        let models = manifest.list_models();
        assert_eq!(models.len(), 2);

        let my_model = models.iter().find(|m| m.name == "my_model").unwrap();
        assert_eq!(my_model.database, "analytics");
        assert_eq!(my_model.columns.len(), 2);
        assert_eq!(my_model.upstream_models, vec!["upstream"]);
        assert_eq!(my_model.tags, vec!["hourly"]);
    }

    #[test]
    fn test_parse_source() {
        let json = r#"{
            "metadata": {},
            "nodes": {},
            "sources": {
                "source.my_project.my_source.my_table": {
                    "name": "my_table",
                    "source_name": "my_source",
                    "database": "raw",
                    "schema": "public",
                    "columns": {
                        "id": {"name": "id", "data_type": "integer"}
                    },
                    "freshness": {
                        "warn_after": {"count": 6, "period": "hours"},
                        "error_after": {"count": 24, "period": "hours"}
                    }
                }
            }
        }"#;
        let manifest = DbtManifest::from_json(json).unwrap();
        let sources = manifest.list_sources();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].name, "my_table");
        assert_eq!(sources[0].database, "raw");
        assert_eq!(sources[0].freshness_warn, Some("6 hours".into()));
    }

    #[test]
    fn test_extract_lineage_edges() {
        let json = r#"{
            "metadata": {},
            "nodes": {
                "model.a.a_model": {
                    "name": "a_model",
                    "resource_type": "model",
                    "database": "db",
                    "schema": "public",
                    "alias": "a_model",
                    "depends_on": {"nodes": ["model.b.base"], "macros": []},
                    "columns": {},
                    "tags": []
                },
                "model.b.base": {
                    "name": "base",
                    "resource_type": "model",
                    "database": "db",
                    "schema": "staging",
                    "alias": "base",
                    "columns": {},
                    "tags": []
                }
            },
            "sources": {}
        }"#;
        let manifest = DbtManifest::from_json(json).unwrap();
        let edges = manifest.extract_lineage_edges();
        assert_eq!(edges.len(), 1);
        assert_eq!(
            edges[0],
            ("base".into(), "a_model".into(), "dbt_model".into())
        );
    }
}
