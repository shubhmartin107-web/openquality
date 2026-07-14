use std::fs;

use crate::client::Client;
use crate::output::{Format, print_value};

pub async fn dbt_parse_manifest(client: &Client, path: &str, format: &Format) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", path, e);
            return;
        }
    };
    let manifest: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error parsing JSON: {}", e);
            return;
        }
    };
    match client.dbt_parse_manifest(manifest).await {
        Ok(value) => print_value(&value, format),
        Err(e) => eprintln!("Error: {}", e),
    }
}

pub async fn ge_translate(client: &Client, path: &str, format: &Format) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", path, e);
            return;
        }
    };
    let suite: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error parsing JSON: {}", e);
            return;
        }
    };
    match client.ge_translate(suite).await {
        Ok(value) => print_value(&value, format),
        Err(e) => eprintln!("Error: {}", e),
    }
}

pub async fn lineage_parse_sql(client: &Client, sql: &str, format: &Format) {
    match client.lineage_parse_sql(sql).await {
        Ok(value) => print_value(&value, format),
        Err(e) => eprintln!("Error: {}", e),
    }
}

pub async fn lineage_build_graph(client: &Client, paths: &[String], format: &Format) {
    let mut statements = Vec::new();
    for path in paths {
        match fs::read_to_string(path) {
            Ok(content) => {
                for stmt in content.split(';') {
                    let trimmed = stmt.trim();
                    if !trimmed.is_empty() {
                        statements.push(trimmed.to_string());
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading file '{}': {}", path, e);
                return;
            }
        }
    }
    match client.lineage_build_graph(&statements).await {
        Ok(value) => print_value(&value, format),
        Err(e) => eprintln!("Error: {}", e),
    }
}
