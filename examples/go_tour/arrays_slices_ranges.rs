//! Go Tour mapping: arrays, slices, slice length/capacity, append, and range.
//!
//! Rust uses fixed arrays, borrowed slices, and `Vec<T>` for growable lists.

use ezrs::{App, Context, Result};

fn sum(values: &[i32]) -> i32 {
    values.iter().sum()
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("list", list).run().await
}

async fn list(ctx: Context) -> Result<()> {
    let array = [1, 2, 3, 4];
    let slice = &array[1..3];
    let mut values = Vec::from(slice);
    values.push(10);

    for (index, value) in values.iter().enumerate() {
        ctx.println(format!("{index}: {value}"));
    }

    ctx.println(format!("sum={}", sum(&values)));
    Ok(())
}
