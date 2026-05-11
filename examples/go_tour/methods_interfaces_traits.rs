//! Go Tour mapping: interfaces.
//!
//! Rust uses traits. Unlike Go, implementation is explicit.

use ezrs::{App, Context, Result};

trait Greeter {
    fn greet(&self) -> String;
}

struct User {
    name: String,
}

impl Greeter for User {
    fn greet(&self) -> String {
        format!("hello {}", self.name)
    }
}

fn render(value: &impl Greeter) -> String {
    value.greet()
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(r#trait).run().await
}

async fn r#trait(ctx: Context) -> Result<()> {
    let user = User {
        name: String::from("Ayu"),
    };
    ctx.println(render(&user));
    Ok(())
}
