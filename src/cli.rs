use clap::Parser;
use std::path::PathBuf;

pub const DEFAULT_PORT: u16 = 5600;
pub const DEFAULT_INTERVAL_SECS: u64 = 60;

#[derive(Parser, Debug)]
#[command(name = "aw-watcher-network")]
pub struct Cli {
    /// Sends events to a testing bucket and does not persist state.
    #[arg(long)]
    pub testing: bool,
    /// Overrides the ActivityWatch server port (default: 5600).
    #[arg(long, default_value_t = DEFAULT_PORT)]
    pub port: u16,
    /// Heartbeat interval in seconds (default: 60s).
    #[arg(long, default_value_t = DEFAULT_INTERVAL_SECS)]
    pub interval: u64,
    /// Optional path to a YAML config with location mappings.
    #[arg(long)]
    pub config: Option<PathBuf>,
}
