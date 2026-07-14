use crate::client::Client;
use crate::output::{Format, print_value};

pub async fn list(client: &Client, ws_id: &str, format: &Format) {
    match client.list_monitors(ws_id).await {
        Ok(value) => print_value(&value, format),
        Err(e) => eprintln!("Error: {}", e),
    }
}

pub async fn create(
    client: &Client,
    ws_id: &str,
    name: &str,
    mon_type: &str,
    table: &str,
    cron: Option<&str>,
    format: &Format,
) {
    match client
        .create_monitor(ws_id, name, mon_type, table, cron)
        .await
    {
        Ok(value) => print_value(&value, format),
        Err(e) => eprintln!("Error: {}", e),
    }
}

pub async fn delete(client: &Client, id: &str, format: &Format) {
    match client.delete_monitor(id).await {
        Ok(value) => print_value(&value, format),
        Err(e) => eprintln!("Error: {}", e),
    }
}

pub async fn run(client: &Client, id: &str, format: &Format) {
    match client.run_monitor(id).await {
        Ok(value) => print_value(&value, format),
        Err(e) => eprintln!("Error: {}", e),
    }
}
