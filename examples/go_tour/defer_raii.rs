//! Go Tour mapping: defer.
//!
//! Rust cleanup usually happens with RAII: values clean up when they leave scope.

use ezrs::{App, Context, Result};

struct Cleanup(&'static str);

impl Drop for Cleanup {
    fn drop(&mut self) {
        eprintln!("cleanup: {}", self.0);
    }
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(work).run().await
}

async fn work(ctx: Context) -> Result<()> {
    let _cleanup = Cleanup("temporary resource");
    ctx.println("working");
    Ok(())
}
