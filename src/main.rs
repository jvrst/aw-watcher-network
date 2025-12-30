use aw_client_rust::AwClient;
use aw_models::Event;
use chrono::{TimeDelta, Utc};
use clap::Parser;
use serde_json::{Map, Value};
use std::thread::sleep;
use std::time::Duration;

mod aw;
mod cli;
mod config;
mod location;
mod net;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();
    let aw_client = AwClient::new("localhost", &cli.port.to_string(), "aw-watcher-network");
    let base_bucket = if cli.testing {
        "aw-watcher-network-testing"
    } else {
        "aw-watcher-network"
    };
    let bucket_id = format!("{}_{}", base_bucket, aw_client.hostname);
    aw::create_bucket(&aw_client, &bucket_id)?;

    let location_rules = config::load_location_rules(&cli.config);
    let interval = Duration::from_secs(cli.interval);

    loop {
        let public_ip = net::fetch_public_ip().unwrap_or_else(|| "unknown".to_string());
        let (gateway_ip, connection_type) = net::detect_gateway_and_type();
        let connection_type = connection_type
            .map(|value| value.as_str().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let gateway_ip = gateway_ip.unwrap_or_else(|| "unknown".to_string());
        let location = location_rules
            .as_ref()
            .and_then(|rules| rules.resolve(&public_ip))
            .unwrap_or_else(|| "unknown".to_string());

        let mut data = Map::new();
        data.insert("public_ip".to_string(), Value::String(public_ip));
        data.insert("gateway_ip".to_string(), Value::String(gateway_ip));
        data.insert(
            "connection_type".to_string(),
            Value::String(connection_type),
        );
        data.insert("location".to_string(), Value::String(location.clone()));
        data.insert("title".to_string(), Value::String(location));

        let event = Event {
            id: None,
            timestamp: Utc::now(),
            duration: TimeDelta::seconds(1),
            data,
        };

        if let Err(err) = aw_client.heartbeat(&bucket_id, &event, cli.interval as f64 + 1.0) {
            eprintln!("Failed to send heartbeat: {err}");
        }
        sleep(interval);
    }
}
