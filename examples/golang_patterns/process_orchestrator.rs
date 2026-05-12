//! Go pattern: exec.CommandContext in a small orchestrator command.

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(check_toolchain).run().await
}

async fn check_toolchain(ctx: Context) -> Result<()> {
    let output = ctx
        .process("rustc")
        .arg("--version")
        .capture()
        .timeout_secs(5)
        .run()
        .await?;

    if output.status.success {
        ctx.println(output.stdout_lossy().trim());
    }

    Ok(())
}
