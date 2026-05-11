//! Go pattern: os.ReadFile, os.WriteFile, and filepath.WalkDir.

use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command("copy", copy).run().await
}

async fn copy(ctx: Context) -> Result<()> {
    let input = ctx.arg_or("input", "input.txt");
    let output = ctx.arg_or("output", "out/output.txt");

    if ctx.fs().exists(&input) {
        let text = ctx.fs().read_to_string(&input).await?;
        ctx.fs().write_string(&output, text).await?;
    }

    for path in ctx.fs().walk(".")? {
        ctx.println(path.display());
    }

    Ok(())
}
