use serde::Deserialize;
use std::path::PathBuf;

use crate::location::LocationRules;

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    pub locations: Option<serde_yaml::Value>,
}

pub fn load_location_rules(path: &Option<PathBuf>) -> Option<LocationRules> {
    let path = path.as_ref()?;
    let contents = std::fs::read_to_string(path).ok()?;
    let config: ConfigFile = serde_yaml::from_str(&contents).ok()?;
    Some(LocationRules::from_config(config))
}
