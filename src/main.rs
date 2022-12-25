use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

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
    /// Include hourly
    #[clap(value_parser, long)]
    pub hourly: Option<bool>,
    /// Include daily
    #[clap(value_parser, long)]
    pub daily: Option<bool>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::new();
    match cli {
        Commands::Query(args) => {
            let mut config = Config::of_file()?;
            if let Some(key) = args.api_key {
                log::warn!("Api key should be set in config file");
                config.api_key = key;
            }
            if let Some(mode) = args.mode {
                log::warn!("Geometry mode should be set in config file");
                config.geometry_mode = mode;
            }
            if let Some(b) = args.hourly {
                config.hourly = b;
            }
            if let Some(b) = args.daily {
                config.daily = b;
            }
            let geo = Geometry::new(&config.geometry_mode).await?;
            let weather = Weather::new(config, geo).await?;
            println!("{}", weather.body);
            Ok(())
        }
        Commands::Edit => {
            use std::process::Command;

            let conf = match Config::of_file() {
                Ok(conf) => conf,
                Err(_) => Config::fresh(),
            };
            conf.to_file()?;

            let editor = std::env::var("EDITOR").or_else(|err| {
                println!("Please set $EDITOR to your preferred editor.");
                Err(err)
            })?;

            let mut child = Command::new(editor)
                .args([Config::config_path()?])
                .spawn()?;
            child.wait()?;
            Ok(())
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Config {
    api_key: String,
    geometry_mode: GeometryMode,
    hourly: bool,
    daily: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, ValueEnum)]
pub enum GeometryMode {
    Location,
    City,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: Default::default(),
            geometry_mode: GeometryMode::Location,
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
    pub fn to_file(&self) -> anyhow::Result<()> {
        let text = toml::to_string(self)?;
        fs::write(&Self::config_path()?, text)
            .map_err(|_| anyhow::anyhow!("error writing config file"))?;
        Ok(())
    }
}

#[derive(Debug)]
enum Geometry {
    Location { lat: f64, lon: f64 },
    City { city: String, country_code: String },
}

impl Geometry {
    async fn new(mode: &GeometryMode) -> anyhow::Result<Self> {
        let url = url::Url::parse("http://ip-api.com/json/")?;
        let text = reqwest::get(url).await?.text().await?;
        let json: serde_json::Value = serde_json::from_str(text.as_str())?;
        log::info!("geo json: {:?}", json);
        fn field_access<T>(
            json: &serde_json::Value,
            field: &'static str,
            f: impl FnOnce(&serde_json::Value) -> Option<T>,
        ) -> anyhow::Result<T> {
            json.get(field)
                .and_then(f)
                .ok_or_else(|| anyhow::anyhow!(format!("error getting {} from ip-api", field)))
        }
        match mode {
            GeometryMode::Location => {
                let field = |field: &'static str| -> anyhow::Result<f64> {
                    field_access(&json, field, |x| x.as_f64())
                };
                let lat = field("lat")?;
                let lon = field("lon")?;
                Ok(Geometry::Location { lat, lon })
            }
            GeometryMode::City => {
                let field = |field: &'static str| -> anyhow::Result<String> {
                    field_access(&json, field, |x| x.as_str().map(ToString::to_string))
                };
                let city = field("city")?;
                let country_code = field("countryCode")?;
                Ok(Geometry::City { city, country_code })
            }
        }
    }
}

struct Weather {
    body: String,
}

impl Weather {
    async fn new(config: Config, geo: Geometry) -> anyhow::Result<Self> {
        let mut url_str = format!("https://api.openweathermap.org/data");
        match geo {
            Geometry::Location { lat, lon } => {
                url_str += &format!("/3.0/onecall");
                url_str += &format!("?appid={}", config.api_key);
                url_str += &format!("&lat={}", lat);
                url_str += &format!("&lon={}", lon);
            }
            Geometry::City { city, country_code } => {
                url_str += &format!("/2.5/weather");
                url_str += &format!("?appid={}", config.api_key);
                url_str += &format!("&q={},{}", city, country_code);
            }
        }
        url_str += &format!("&units={}", "metric");
        url_str += &format!("&exclude=minutely");
        if !config.hourly {
            url_str += &format!(",hourly");
        }
        if !config.daily {
            url_str += &format!(",daily");
        }
        log::info!("weather url: {}", url_str);
        let url = url::Url::parse(&url_str)?;
        let body = reqwest::get(url).await?.text().await?;
        Ok(Self { body })
    }
}
