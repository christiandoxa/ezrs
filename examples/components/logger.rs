//! Go pattern: boring default logger.

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(work).run().await
}

async fn work(ctx: Context) -> Result<()> {
    ctx.log().info("started");
    ctx.log()
        .info_fields("worker ready", [("worker", "scan"), ("queue", "default")]);
    ctx.log().warn("skipped optional file");
    ctx.log().error("example error log");
    ctx.println("set EZRS_LOG=debug or RUST_LOG=debug for more logs");
    Ok(())
}
