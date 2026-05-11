# Lifecycle

`Lifecycle` is the small ezrs building block for Go-style startup, readiness, and graceful shutdown patterns.

It keeps hooks explicit and syntax-first:

```rust
use ezrs::{App, Context, Result};

async fn load_config(ctx: Context) -> Result<()> {
    ctx.println("config loaded");
    Ok(())
}

async fn announce_ready(ctx: Context) -> Result<()> {
    ctx.println("ready");
    Ok(())
}

async fn flush_state(ctx: Context) -> Result<()> {
    ctx.println("state flushed");
    Ok(())
}

async fn run(ctx: Context) -> Result<()> {
    ctx.println("running");
    Ok(())
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new()
        .on_start(load_config)
        .on_ready(announce_ready)
        .on_shutdown(flush_state)
        .shutdown_timeout_secs(10)
        .command(run)
        .run()
        .await
}
```

## Order

Startup hooks run in registration order.

Readiness hooks run in registration order.

Shutdown hooks run in reverse registration order. This mirrors stack-style cleanup: the last resource started is the first resource stopped.

## Timeout

`shutdown_timeout_secs` bounds the whole shutdown phase. If hooks do not finish before the timeout, `run_shutdown` returns a timeout error.

## API

- `Lifecycle::new()`
- `Lifecycle::on_start(handler)`
- `Lifecycle::on_ready(handler)`
- `Lifecycle::on_shutdown(handler)`
- `Lifecycle::shutdown_timeout_secs(seconds)`
- `Lifecycle::run_start(ctx)`
- `Lifecycle::run_ready(ctx)`
- `Lifecycle::run_shutdown(ctx)`
- `LifecycleHook::new(handler)`
- `LifecycleHook::named(name, handler)`

Handlers are async functions or closures that take `Context` and return `ezrs::Result<()>`. Futures are boxed internally, so this does not require `async-trait`.
