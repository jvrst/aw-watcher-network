use default_net::get_default_interface;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ConnectionType {
    Wifi,
    Lan,
}

impl ConnectionType {
    pub fn as_str(self) -> &'static str {
        match self {
            ConnectionType::Wifi => "wifi",
            ConnectionType::Lan => "lan",
        }
    }
}

pub fn fetch_public_ip() -> Option<String> {
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

pub fn detect_gateway_and_type() -> (Option<String>, Option<ConnectionType>) {
    let Ok(interface) = get_default_interface() else {
        return (None, None);
    };
    let gateway_ip = interface
        .gateway
        .map(|gateway| gateway.ip_addr.to_string());
    let name = interface.name.clone();
    let connection_type = classify_connection_type(&name);
    (gateway_ip, connection_type)
}

pub(crate) fn classify_connection_type(name: &str) -> Option<ConnectionType> {
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

#[cfg(test)]
mod tests {
    use super::{classify_connection_type, ConnectionType};

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_classification() {
        assert_eq!(
            classify_connection_type("Wi-Fi"),
            Some(ConnectionType::Wifi)
        );
        assert_eq!(
            classify_connection_type("WLAN Adapter"),
            Some(ConnectionType::Wifi)
        );
        assert_eq!(
            classify_connection_type("Ethernet 2"),
            Some(ConnectionType::Lan)
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_classification() {
        assert_eq!(classify_connection_type("en0"), Some(ConnectionType::Wifi));
        assert_eq!(
            classify_connection_type("awdl0"),
            Some(ConnectionType::Wifi)
        );
        assert_eq!(classify_connection_type("en2"), Some(ConnectionType::Lan));
        assert_eq!(classify_connection_type("en5"), Some(ConnectionType::Lan));
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    #[test]
    fn linux_classification() {
        assert_eq!(
            classify_connection_type("wlan0"),
            Some(ConnectionType::Wifi)
        );
        assert_eq!(classify_connection_type("wlx123"), Some(ConnectionType::Wifi));
        assert_eq!(
            classify_connection_type("eth0"),
            Some(ConnectionType::Lan)
        );
        assert_eq!(
            classify_connection_type("enp3s0"),
            Some(ConnectionType::Lan)
        );
        assert_eq!(
            classify_connection_type("eno1"),
            Some(ConnectionType::Lan)
        );
        assert_eq!(
            classify_connection_type("ens33"),
            Some(ConnectionType::Lan)
        );
    }
}
