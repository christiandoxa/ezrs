//! Go Tour mapping: type parameters.
//!
//! Rust generics use `<T>` and trait bounds.

use ezrs::{App, Context, Result};

fn first<T: Clone>(items: &[T]) -> Option<T> {
    items.first().cloned()
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("generic", generic).run().await
}

async fn generic(ctx: Context) -> Result<()> {
    let values = vec![10, 20, 30];
    ctx.println(first(&values).unwrap_or_default());
    Ok(())
}
