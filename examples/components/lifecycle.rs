//! Generic lifecycle hooks for startup, readiness, and graceful shutdown.

use ezrs::{App, Context, Result};

async fn load_config(ctx: Context) -> Result<()> {
    ctx.println("config loaded");
    Ok(())
}

async fn verify_ready(ctx: Context) -> Result<()> {
    ctx.println("ready");
    Ok(())
}

async fn flush_state(ctx: Context) -> Result<()> {
    ctx.println("state flushed");
    Ok(())
}

async fn run(ctx: Context) -> Result<()> {
    ctx.println("running");
    Ok(())
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new()
        .on_start(load_config)
        .on_ready(verify_ready)
        .on_shutdown(flush_state)
        .shutdown_timeout_secs(5)
        .command(run)
        .run()
        .await
}
