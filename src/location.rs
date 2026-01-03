use ipnet::IpNet;
use std::net::IpAddr;
use std::str::FromStr;

use crate::config::ConfigFile;

pub struct LocationRules {
    ordered: Vec<(String, Vec<String>)>,
}

impl LocationRules {
    pub fn from_config(config: ConfigFile) -> Self {
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

    pub fn resolve(&self, public_ip: &str) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use super::LocationRules;
    use crate::config::ConfigFile;

    #[test]
    fn resolves_exact_ip() {
        let config = ConfigFile {
            locations: Some(serde_yaml::from_str(
                "home:\n  - 1.2.3.4\n",
            ).unwrap()),
        };
        let rules = LocationRules::from_config(config);
        assert_eq!(rules.resolve("1.2.3.4").as_deref(), Some("home"));
        assert_eq!(rules.resolve("1.2.3.5"), None);
    }

    #[test]
    fn resolves_cidr() {
        let config = ConfigFile {
            locations: Some(serde_yaml::from_str(
                "office:\n  - 10.20.0.0/16\n",
            ).unwrap()),
        };
        let rules = LocationRules::from_config(config);
        assert_eq!(rules.resolve("10.20.5.6").as_deref(), Some("office"));
        assert_eq!(rules.resolve("10.21.5.6"), None);
    }

    #[test]
    fn resolves_first_match_only() {
        let config = ConfigFile {
            locations: Some(serde_yaml::from_str(
                "home:\n  - 10.0.0.0/8\noffice:\n  - 10.0.0.0/8\n",
            ).unwrap()),
        };
        let rules = LocationRules::from_config(config);
        assert_eq!(rules.resolve("10.1.2.3").as_deref(), Some("home"));
    }

    #[test]
    fn resolves_multiple_entries_ipv4() {
        let config = ConfigFile {
            locations: Some(serde_yaml::from_str(
                "office:\n  - 192.0.2.10\n  - 192.0.2.20\n",
            ).unwrap()),
        };
        let rules = LocationRules::from_config(config);
        assert_eq!(
            rules.resolve("192.0.2.10").as_deref(),
            Some("office")
        );
        assert_eq!(
            rules.resolve("192.0.2.20").as_deref(),
            Some("office")
        );
    }

    #[test]
    fn resolves_ipv6_exact() {
        let config = ConfigFile {
            locations: Some(serde_yaml::from_str(
                "office:\n  - 2001:db8::1\n",
            ).unwrap()),
        };
        let rules = LocationRules::from_config(config);
        assert_eq!(
            rules.resolve("2001:db8::1").as_deref(),
            Some("office")
        );
        assert_eq!(rules.resolve("2001:db8::2"), None);
    }
}
