//! Go Tour mapping: methods and pointer receivers.
//!
//! Rust implements methods with `impl` blocks. `&mut self` maps to mutable receivers.

use ezrs::{App, Context, Result};

struct Counter {
    value: u64,
}

impl Counter {
    fn increment(&mut self) {
        self.value += 1;
    }
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(method).run().await
}

async fn method(ctx: Context) -> Result<()> {
    let mut counter = Counter { value: 0 };
    counter.increment();
    ctx.println(counter.value);
    Ok(())
}
