use crate::client::Client;
use crate::output::{Format, print_value};

pub async fn run(client: &Client, format: &Format) {
    match client.health().await {
        Ok(value) => print_value(&value, format),
        Err(e) => eprintln!("Error: {}", e),
    }
}
