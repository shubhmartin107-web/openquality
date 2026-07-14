use crate::client::Client;
use crate::output::{Format, print_value};

pub async fn list(client: &Client, ws_id: &str, format: &Format) {
    match client.list_data_sources(ws_id).await {
        Ok(value) => print_value(&value, format),
        Err(e) => eprintln!("Error: {}", e),
    }
}

pub async fn create(
    client: &Client,
    ws_id: &str,
    name: &str,
    conn_type: &str,
    conn_str: &str,
    format: &Format,
) {
    match client
        .create_data_source(ws_id, name, conn_type, conn_str)
        .await
    {
        Ok(value) => print_value(&value, format),
        Err(e) => eprintln!("Error: {}", e),
    }
}

pub async fn delete(client: &Client, id: &str, format: &Format) {
    match client.delete_data_source(id).await {
        Ok(value) => print_value(&value, format),
        Err(e) => eprintln!("Error: {}", e),
    }
}
