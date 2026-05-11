//! Go Tour mapping: goroutines.
//!
//! ezrs maps app-level goroutine-style work to named Tokio tasks through `ctx.spawn`.

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("tasks", tasks).run().await
}

async fn tasks(ctx: Context) -> Result<()> {
    for id in 0..3 {
        let worker_ctx = ctx.clone();
        ctx.spawn(format!("worker-{id}"), async move {
            worker_ctx.println(format!("worker {id}"));
            Ok(())
        });
    }

    ctx.join_all().await
}
