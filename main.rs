use aw_client_rust::AwClient;
use aw_models::Event;
use chrono::{TimeDelta, Utc};
use clap::Parser;
use default_net::get_default_interface;
use ipnet::IpNet;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

const DEFAULT_PORT: u16 = 5600;
const DEFAULT_INTERVAL_SECS: u64 = 60;

#[derive(Parser, Debug)]
#[command(name = "aw-watcher-network")]
struct Cli {
    /// Sends events to a testing bucket and does not persist state.
    #[arg(long)]
    testing: bool,
    /// Overrides the ActivityWatch server port (default: 5600).
    #[arg(long, default_value_t = DEFAULT_PORT)]
    port: u16,
    /// Heartbeat interval in seconds (default: 60s).
    #[arg(long, default_value_t = DEFAULT_INTERVAL_SECS)]
    interval: u64,
    /// Optional path to a YAML config with location mappings.
    #[arg(long)]
    config: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    locations: Option<serde_yaml::Value>,
}

struct LocationRules {
    ordered: Vec<(String, Vec<String>)>,
}

impl LocationRules {
    fn from_config(config: ConfigFile) -> Self {
        let mut ordered = Vec::new();
        if let Some(serde_yaml::Value::Mapping(map)) = config.locations {
            for (key, value) in map {
                let Some(name) = key.as_str().map(|s| s.to_string()) else {
                    continue;
                };
                let mut entries = Vec::new();
                if let serde_yaml::Value::Sequence(seq) = value {
                    for item in seq {
                        if let Some(item_str) = item.as_str() {
                            entries.push(item_str.to_string());
                        }
                    }
                }
                ordered.push((name, entries));
            }
        }
        Self { ordered }
    }

    fn resolve(&self, public_ip: &str) -> Option<String> {
        let ip = IpAddr::from_str(public_ip).ok()?;
        for (location, entries) in &self.ordered {
            for entry in entries {
                if let Ok(net) = IpNet::from_str(entry) {
                    if net.contains(&ip) {
                        return Some(location.clone());
                    }
                    continue;
                }
                if let Ok(single_ip) = IpAddr::from_str(entry) {
                    if single_ip == ip {
                        return Some(location.clone());
                    }
                }
            }
        }
        None
    }
}

fn create_bucket(aw_client: &AwClient, bucket_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    aw_client.create_bucket(bucket_id, "network")?;
    Ok(())
}

fn fetch_public_ip() -> Option<String> {
    let client = reqwest::blocking::Client::new();
    let endpoints = [
        "https://api64.ipify.org?format=json",
        "https://api.ipify.org?format=json",
    ];
    for endpoint in endpoints {
        let resp = match client.get(endpoint).send() {
            Ok(resp) => resp,
            Err(_) => continue,
        };
        let payload: serde_json::Value = match resp.json() {
            Ok(payload) => payload,
            Err(_) => continue,
        };
        if let Some(ip) = payload.get("ip").and_then(|value| value.as_str()) {
            return Some(ip.to_string());
        }
    }
    None
}

fn detect_gateway_and_type() -> (Option<String>, Option<String>) {
    let Ok(interface) = get_default_interface() else {
        return (None, None);
    };
    let gateway_ip = interface
        .gateway
        .map(|gateway| gateway.ip_addr.to_string());
    let name = interface.name.clone();
    let connection_type = classify_connection_type(&name).map(|value| value.as_str().to_string());
    (gateway_ip, connection_type)
}

#[derive(Copy, Clone, Debug)]
enum ConnectionType {
    Wifi,
    Lan,
}

impl ConnectionType {
    fn as_str(self) -> &'static str {
        match self {
            ConnectionType::Wifi => "wifi",
            ConnectionType::Lan => "lan",
        }
    }
}

fn classify_connection_type(name: &str) -> Option<ConnectionType> {
    classify_connection_type_impl(name)
}

#[cfg(target_os = "windows")]
fn classify_connection_type_impl(name: &str) -> Option<ConnectionType> {
    let lower = name.to_ascii_lowercase();
    if lower.contains("wlan") || lower.contains("wi-fi") || lower.contains("wifi") {
        return Some(ConnectionType::Wifi);
    }
    if lower.contains("ethernet") {
        return Some(ConnectionType::Lan);
    }
    None
}

#[cfg(target_os = "macos")]
fn classify_connection_type_impl(name: &str) -> Option<ConnectionType> {
    if name == "en0" || name.starts_with("awdl") {
        return Some(ConnectionType::Wifi);
    }
    if name.starts_with("en") {
        return Some(ConnectionType::Lan);
    }
    None
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn classify_connection_type_impl(name: &str) -> Option<ConnectionType> {
    let lower = name.to_ascii_lowercase();
    if lower.starts_with("wlan") || lower.starts_with("wl") {
        return Some(ConnectionType::Wifi);
    }
    if lower.starts_with("eth")
        || lower.starts_with("enp")
        || lower.starts_with("eno")
        || lower.starts_with("ens")
    {
        return Some(ConnectionType::Lan);
    }
    None
}

fn load_location_rules(path: &Option<PathBuf>) -> Option<LocationRules> {
    let path = path.as_ref()?;
    let contents = std::fs::read_to_string(path).ok()?;
    let config: ConfigFile = serde_yaml::from_str(&contents).ok()?;
    Some(LocationRules::from_config(config))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let aw_client = AwClient::new("localhost", &cli.port.to_string(), "aw-watcher-network");
    let base_bucket = if cli.testing {
        "aw-watcher-network-testing"
    } else {
        "aw-watcher-network"
    };
    let bucket_id = format!("{}_{}", base_bucket, aw_client.hostname);
    create_bucket(&aw_client, &bucket_id)?;

    let location_rules = load_location_rules(&cli.config);
    let interval = Duration::from_secs(cli.interval);

    loop {
        let public_ip = fetch_public_ip().unwrap_or_else(|| "unknown".to_string());
        let (gateway_ip, connection_type) = detect_gateway_and_type();
        let connection_type = connection_type.unwrap_or_else(|| "unknown".to_string());
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
        data.insert("location".to_string(), Value::String(location));

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
