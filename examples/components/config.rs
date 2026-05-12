//! Go pattern: typed config struct loaded at startup.

use ezrs::{App, ConfigSource, Context, Error, Result};

#[derive(Clone, serde::Deserialize)]
struct Config {
    database_url: String,
    workers: usize,
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new()
        .config_validated::<Config, _>(
            ConfigSource::ezrs().env_prefix("APP"),
            |config| {
                if config.workers == 0 {
                    Err(Error::invalid_input("workers must be greater than zero"))
                } else {
                    Ok(())
                }
            },
        )
        .command(run)
        .run()
        .await
}

async fn run(ctx: Context) -> Result<()> {
    let cfg = ctx.config::<Config>()?;
    ctx.println(format!("database: {}", cfg.database_url));
    ctx.println(format!("workers: {}", cfg.workers));
    Ok(())
}

// ezrs.toml:
// database_url = "postgres://localhost/app"
// workers = 4
//
// Env overlay:
// APP_WORKERS=8 cargo run -- run
