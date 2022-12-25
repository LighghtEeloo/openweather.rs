use crate::config::{Config, GeometryMode};

#[derive(Debug)]
pub enum Geometry {
    Location { lat: f64, lon: f64 },
    City { city: String, country_code: String },
}

impl Geometry {
    pub async fn new(mode: &GeometryMode) -> anyhow::Result<Self> {
        let url = url::Url::parse("http://ip-api.com/json/")?;
        let text = reqwest::get(url).await?.text().await?;
        let json: serde_json::Value = serde_json::from_str(text.as_str())?;
        log::info!("[geo json]: {:?}", json);
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

pub struct Weather {
    pub body: String,
}

impl Weather {
    pub async fn new(config: Config, geo: Geometry) -> anyhow::Result<Self> {
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
        let mut exclude = Vec::new();
        if !config.minutely {
            exclude.push("minutely");
        }
        if !config.hourly {
            exclude.push("hourly");
        }
        if !config.daily {
            exclude.push("daily");
        }
        if exclude.len() > 0 {
            url_str += &format!("&exclude={}", exclude.join(","));
        }
        log::info!("[weather url]: {}", url_str);
        let url = url::Url::parse(&url_str)?;
        let body = reqwest::get(url).await?.text().await?;
        Ok(Self { body })
    }
}
