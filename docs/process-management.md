# Process Management

`Process` is the small ezrs building block for running child processes with
timeout-aware and Context-aware cancellation.

```rust
use ezrs::{Context, Result};

async fn check_tool(ctx: Context) -> Result<()> {
    let output = ctx.process("rustc")
        .arg("--version")
        .timeout_secs(5)
        .capture()
        .run()
        .await?;

    if output.status.success {
        ctx.println(output.stdout_lossy().trim());
    }

    Ok(())
}
```

This maps to the `exec.CommandContext` pattern: configure the command, run it
asynchronously, and let the timeout or parent `Context` kill the child if it
does not exit in time.

`ctx.process("program")` attaches the app cancellation token and enables
kill-on-drop. `Process::new("program")` remains the low-level escape hatch for
detached process setup.
