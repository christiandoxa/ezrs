//! Go Tour mapping: functions, multiple results, and named returns.
//!
//! Rust uses typed functions, tuples for multiple values, and structs when names matter.

use ezrs::{App, Context, Result};

#[derive(Debug)]
struct ParsedName {
    first: String,
    last: String,
}

fn split_name(input: &str) -> (&str, &str) {
    input.split_once(' ').unwrap_or((input, ""))
}

fn parse_name(input: &str) -> ParsedName {
    let (first, last) = split_name(input);
    ParsedName {
        first: first.to_owned(),
        last: last.to_owned(),
    }
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("parse", parse).run().await
}

async fn parse(ctx: Context) -> Result<()> {
    let name = parse_name(&ctx.arg_or("name", "Ada Lovelace"));
    ctx.println(format!("first={} last={}", name.first, name.last));
    Ok(())
}
