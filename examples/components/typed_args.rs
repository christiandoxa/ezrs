//! Go pattern: flag package or cobra flags, mapped into a typed Rust struct.

use ezrs::typed_args::{self, ArgSource, TypedArgs};
use ezrs::{App, Context, Result};

struct ScanArgs {
    path: String,
    recursive: bool,
    limit: usize,
    first_input: Option<String>,
}

impl TypedArgs for ScanArgs {
    fn from_source<S>(source: &S) -> Result<Self>
    where
        S: ArgSource + ?Sized,
    {
        Ok(Self {
            path: typed_args::string_or(source, "path", "."),
            recursive: typed_args::flag(source, "recursive"),
            limit: typed_args::value_or(source, "limit", 100)?,
            first_input: typed_args::optional(source, "0")?,
        })
    }
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(scan).run().await
}

async fn scan(ctx: Context) -> Result<()> {
    let args = ScanArgs::from_context(&ctx)?;

    ctx.println(format!("recursive={}", args.recursive));
    ctx.println(format!("path={}", args.path));
    ctx.println(format!("limit={}", args.limit));
    ctx.println(format!(
        "first input={}",
        args.first_input.as_deref().unwrap_or("none")
    ));

    Ok(())
}

// Try:
// cargo run --example typed_args -- scan --recursive --path src --limit=25 input.txt
