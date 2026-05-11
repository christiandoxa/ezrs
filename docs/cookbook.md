# Cookbook

Small recipes for common ezrs application patterns.

## main and run

Use `#[ezrs::main]` for Tokio setup and return `ezrs::Result<()>`.

```rust
use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(run).run().await
}

async fn run(ctx: Context) -> Result<()> {
    ctx.println("hello");
    Ok(())
}
```

## exec.CommandContext

Use `Context::process` for command construction and `timeout_secs` for a
CommandContext-style deadline.

```rust
use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(version).run().await
}

async fn version(ctx: Context) -> Result<()> {
    let output = ctx
        .process("rustc")
        .arg("--version")
        .timeout_secs(5)
        .capture()
        .run()
        .await?;

    ctx.println(output.stdout_lossy().trim());
    Ok(())
}
```

## Worker Pool

Use `WorkerPool::new(worker_fn)` when a worker function can define identity in
Rust syntax. Use a closure when the worker needs cloned state.

```rust
use ezrs::{Result, WorkerPool};

#[ezrs::main]
async fn main() -> Result<()> {
    WorkerPool::new(process_job)
        .workers(4)
        .buffer(32)
        .run(0..100)
        .await
}

async fn process_job(job: u64) -> Result<()> {
    println!("processed {job}");
    Ok(())
}
```

## Fake Repo Test

Keep domain code behind traits so tests can pass fake repositories as state.

```rust
use std::sync::Arc;

use ezrs::{App, Context, Result};

trait Users: Send + Sync {
    fn count(&self) -> usize;
}

#[derive(Clone)]
struct Repo(Arc<dyn Users>);

async fn report(ctx: Context) -> Result<()> {
    let repo = ctx.state::<Repo>()?;
    ctx.println(format!("users: {}", repo.0.count()));
    Ok(())
}

struct FakeUsers;

impl Users for FakeUsers {
    fn count(&self) -> usize {
        3
    }
}

#[ezrs::test]
async fn reports_user_count() {
    let app = App::new()
        .state(Repo(Arc::new(FakeUsers)))
        .command(report);

    let result = app.test().args(["report"]).run().await;
    result.assert_success();
    result.assert_stdout_contains("users: 3");
}
```

## Config and Env

Load typed config at startup and read environment variables from `Context` when
the value is external process state.

```rust
use ezrs::{App, Context, Result};

#[derive(Clone, serde::Deserialize)]
struct Config {
    database_url: String,
    workers: usize,
}

#[ezrs::main]
async fn main() -> Result<()> {
    App::new()
        .config::<Config>()
        .command(run)
        .run()
        .await
}

async fn run(ctx: Context) -> Result<()> {
    let cfg = ctx.config::<Config>()?;
    let home = ctx.env("HOME")?;
    ctx.println(format!("database: {}", cfg.database_url));
    ctx.println(format!("workers: {}", cfg.workers));
    ctx.println(format!("home: {home}"));
    Ok(())
}
```

## Graceful Shutdown

Use the context cancellation handle to stop loops. `App::run` installs Ctrl+C
handling for command contexts.

```rust
use ezrs::{App, Context, Result};

#[ezrs::main]
async fn main() -> Result<()> {
    App::new().command(run).run().await
}

async fn run(ctx: Context) -> Result<()> {
    let group = ctx.task_group().cancel_on_error(true);
    let worker_ctx = ctx.clone();

    group.spawn(async move {
        loop {
            worker_ctx.check_cancelled()?;
            worker_ctx.sleep_secs(1).await;
            worker_ctx.println("tick");
        }
    });

    ctx.cancelled().await;
    group.cancellation().cancel();
    group.join().await
}
```

## Channels and Select

Use `channel` when ezrs error mapping reduces boilerplate. Use `as_tokio` or
`into_tokio` when the full Tokio channel API is needed.

```rust
use ezrs::{Result, channel, select_recv2};

#[ezrs::main]
async fn main() -> Result<()> {
    let (jobs_tx, mut jobs_rx) = channel(8);
    let (signals_tx, mut signals_rx) = channel(1);

    jobs_tx.send("build").await?;
    signals_tx.send("reload").await?;

    match select_recv2(&mut jobs_rx, &mut signals_rx).await {
        ezrs::Select2::Left(job) => println!("job: {job}"),
        ezrs::Select2::Right(signal) => println!("signal: {signal}"),
        ezrs::Select2::Closed => println!("closed"),
    }

    Ok(())
}
```
