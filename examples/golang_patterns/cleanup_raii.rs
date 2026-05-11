//! Go pattern: defer cleanup maps to RAII and Drop.

use ezrs::{App, Context, Result};

struct Cleanup;

impl Drop for Cleanup {
    fn drop(&mut self) {
        eprintln!("cleanup ran");
    }
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(work).run().await
}

async fn work(ctx: Context) -> Result<()> {
    let _cleanup = Cleanup;
    ctx.println("work runs before cleanup");
    Ok(())
}
