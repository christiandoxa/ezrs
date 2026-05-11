//! Go pattern: typed config struct loaded at startup.

use ezrs::{App, Context, Result};

#[derive(Clone, serde::Deserialize)]
struct Config {
    database_url: String,
    workers: usize,
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new()
        .config::<Config>()
        .command("run", run)
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
