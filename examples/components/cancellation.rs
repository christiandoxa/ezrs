//! Go pattern: context cancellation and graceful shutdown.

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(watch).run().await
}

async fn watch(ctx: Context) -> Result<()> {
    let stopper = ctx.clone();
    ctx.spawn(async move {
        stopper.sleep_secs(1).await;
        stopper.cancel();
        Ok(())
    });

    loop {
        if ctx.is_cancelled() {
            ctx.println("shutting down");
            break;
        }

        ctx.check_cancelled()?;
        ctx.println("tick");
        ctx.sleep_secs(1).await;
    }

    ctx.join_all().await?;
    Ok(())
}
