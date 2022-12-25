use crate::config::{Config, GeometryMode};
use clap::{Args, Parser, Subcommand};

/// A command line dictionary
#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub commands: Commands,
}

impl Cli {
    pub fn new() -> Commands {
        Self::parse().commands
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Query weather
    #[clap(aliases = &["q"])]
    Query(QueryArgs),
    // Query(QueryArgs),
    /// Edit the configuration file
    #[clap(aliases = &["e", "config", "init"])]
    Edit,
}

#[derive(Args, Debug)]
pub struct QueryArgs {
    /// Api key (should be set in config file)
    #[clap(value_parser, long)]
    pub api_key: Option<String>,
    /// Geometry mode (should be set in config file)
    #[clap(value_parser, long)]
    pub mode: Option<GeometryMode>,
    /// Include minutely
    #[clap(value_parser, long)]
    pub minutely: Option<bool>,
    /// Include hourly
    #[clap(value_parser, long)]
    pub hourly: Option<bool>,
    /// Include daily
    #[clap(value_parser, long)]
    pub daily: Option<bool>,
}

impl QueryArgs {
    pub fn update_config(self, config: &mut Config) {
        if let Some(key) = self.api_key {
            log::warn!("Api key should be set in config file");
            config.api_key = key;
        }
        if let Some(mode) = self.mode {
            log::warn!("Geometry mode should be set in config file");
            config.geometry_mode = mode;
        }
        if let Some(b) = self.minutely {
            config.minutely = b;
        }
        if let Some(b) = self.hourly {
            config.hourly = b;
        }
        if let Some(b) = self.daily {
            config.daily = b;
        }
    }
}
