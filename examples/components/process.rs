//! Go pattern: exec.CommandContext.

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(version).run().await
}

async fn version(ctx: Context) -> Result<()> {
    let output = ctx
        .process("rustc")
        .arg("--version")
        .timeout_secs(5)
        .capture()
        .run()
        .await?;

    if output.status.success {
        ctx.println(output.stdout_lossy().trim());
    }

    Ok(())
}
