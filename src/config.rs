use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub api_key: String,
    pub geometry_mode: GeometryMode,
    #[serde(default)]
    pub minutely: bool,
    #[serde(default)]
    pub hourly: bool,
    #[serde(default)]
    pub daily: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, ValueEnum)]
pub enum GeometryMode {
    #[serde(rename = "location", alias = "Location", alias = "v2.5")]
    Location,
    #[serde(rename = "city", alias = "City", alias = "v3.0")]
    City,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: Default::default(),
            geometry_mode: GeometryMode::Location,
            minutely: false,
            hourly: false,
            daily: false,
        }
    }
}

impl Config {
    fn config_path() -> anyhow::Result<PathBuf> {
        use directories_next::ProjectDirs;
        let project_dirs = ProjectDirs::from("", "LitiaEeloo", "OpenWeather")
            .ok_or_else(|| anyhow::anyhow!("No valid config directory fomulated"))?;
        let config_file = "config.toml";
        let mut config_path = project_dirs.config_dir().to_owned();
        fs::create_dir_all(&config_path)?;
        config_path.push(config_file);
        Ok(config_path)
    }
    pub fn of_file() -> anyhow::Result<Self> {
        let config_text = fs::read_to_string(&Self::config_path()?).map_err(|_| {
            anyhow::anyhow!("error opening config file; did you run `open-weather init`?")
        })?;
        let config: Self = toml::from_str(&config_text)?;
        if config.api_key.is_empty() {
            Err(anyhow::anyhow!("empty key"))?
        }
        Ok(config)
    }
    pub fn fresh() -> Self {
        Config::default()
    }
    pub fn edit() -> anyhow::Result<()> {
        let editor = std::env::var("EDITOR").or_else(|err| {
            println!("Please set $EDITOR to your preferred editor.");
            Err(err)
        })?;
        let mut child = std::process::Command::new(editor)
            .args([Config::config_path()?])
            .spawn()?;
        child.wait()?;
        Ok(())
    }
    pub fn to_file(&self) -> anyhow::Result<()> {
        let text = toml::to_string(self)?;
        fs::write(&Self::config_path()?, text)
            .map_err(|_| anyhow::anyhow!("error writing config file"))?;
        Ok(())
    }
}
