use crate::client::Client;
use crate::output::{Format, print_value};

pub async fn list(client: &Client, format: &Format) {
    match client.list_incidents().await {
        Ok(value) => print_value(&value, format),
        Err(e) => eprintln!("Error: {}", e),
    }
}

pub async fn get(client: &Client, id: &str, format: &Format) {
    match client.get_incident(id).await {
        Ok(value) => print_value(&value, format),
        Err(e) => eprintln!("Error: {}", e),
    }
}

pub async fn acknowledge(client: &Client, id: &str, format: &Format) {
    match client.acknowledge_incident(id).await {
        Ok(value) => print_value(&value, format),
        Err(e) => eprintln!("Error: {}", e),
    }
}

pub async fn resolve(client: &Client, id: &str, format: &Format) {
    match client.resolve_incident(id).await {
        Ok(value) => print_value(&value, format),
        Err(e) => eprintln!("Error: {}", e),
    }
}
