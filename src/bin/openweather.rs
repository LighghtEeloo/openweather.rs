use openweather_cli::{
    cli::{Cli, Commands},
    config::Config,
    query::{Geometry, Weather},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::new();
    match cli {
        Commands::Query(args) => {
            let mut config = Config::of_file()?;
            args.update_config(&mut config);
            let geo = Geometry::new(&config.geometry_mode).await?;
            let weather = Weather::new(config, geo).await?;
            println!("{}", weather.body);
            Ok(())
        }
        Commands::Edit => {
            match Config::of_file() {
                Ok(_) => (),
                Err(_) => Config::fresh().to_file()?,
            };
            Config::edit()?;
            Ok(())
        }
    }
}
