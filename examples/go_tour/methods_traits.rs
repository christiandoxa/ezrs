//! Go Tour mapping: methods, pointer receivers, and interfaces.
//!
//! Rust uses inherent `impl` methods and traits. `&mut self` maps to mutation through a receiver.

use ezrs::{App, Context, Result};

struct Counter {
    value: u64,
}

impl Counter {
    fn new() -> Self {
        Self { value: 0 }
    }

    fn add(&mut self, value: u64) {
        self.value += value;
    }
}

trait Reporter {
    fn report(&self) -> String;
}

impl Reporter for Counter {
    fn report(&self) -> String {
        format!("counter={}", self.value)
    }
}

fn render_report(item: &dyn Reporter) -> String {
    item.report()
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(methods).run().await
}

async fn methods(ctx: Context) -> Result<()> {
    let mut counter = Counter::new();
    counter.add(2);
    counter.add(3);
    ctx.println(render_report(&counter));
    Ok(())
}
