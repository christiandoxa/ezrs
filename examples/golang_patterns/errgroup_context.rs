//! Go pattern: errgroup.WithContext.

use ezrs::{Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    ezrs::App::new().command(run).run().await
}

async fn run(ctx: Context) -> Result<()> {
    let group = ctx.err_group();

    for item in ["a", "b", "c"] {
        group.spawn_named_with_cancel(item, move |cancellation| async move {
            cancellation.check_cancelled()?;
            println!("processed {item}");
            Ok(())
        });
    }

    group.join().await
}
