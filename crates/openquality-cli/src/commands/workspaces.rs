use crate::client::Client;
use crate::output::{Format, print_value};

pub async fn list(client: &Client, format: &Format) {
    match client.list_workspaces().await {
        Ok(value) => print_value(&value, format),
        Err(e) => eprintln!("Error: {}", e),
    }
}

pub async fn create(client: &Client, name: &str, slug: &str, format: &Format) {
    match client.create_workspace(name, slug).await {
        Ok(value) => print_value(&value, format),
        Err(e) => eprintln!("Error: {}", e),
    }
}
