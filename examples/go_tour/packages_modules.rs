//! Go Tour mapping: packages, imports, and exported names.
//!
//! Rust uses crates and modules. Public items use `pub`, not uppercase names.

use ezrs::{App, Context, Result};

mod greetings {
    pub struct Greeter {
        prefix: String,
    }

    impl Greeter {
        pub fn new(prefix: impl Into<String>) -> Self {
            Self {
                prefix: prefix.into(),
            }
        }

        pub fn greet(&self, name: &str) -> String {
            format!("{} {name}", self.prefix)
        }
    }
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new()
        .name("go-tour-packages")
        .command(hello)
        .run()
        .await
}

async fn hello(ctx: Context) -> Result<()> {
    let greeter = greetings::Greeter::new("hello");
    ctx.println(greeter.greet(&ctx.arg_or("name", "gopher")));
    Ok(())
}
