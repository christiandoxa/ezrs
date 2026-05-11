# ezrs

ezrs - Go-style application patterns, Rust-grade safety.

ezrs helps Go developers build familiar application architecture in idiomatic Rust. It is not a Go syntax clone, runtime, standard library clone, or translator. It provides a small Rust-native framework for CLI tools, workers, file processors, automation tools, small daemons, and background jobs.

## Product goal

ezrs answers: "How do I build the same kind of application architecture I use in Go, but in proper Rust?"

It maps Go application patterns like `run() error`, `context.Context`, explicit dependencies, goroutines, channels, cancellation, worker pools, simple config structs, boring logging, and table-driven tests into safe Rust APIs and examples.

## Installation

Library users normally depend only on the facade crate:

```toml
[dependencies]
ezrs = "0.1.0"
serde = { version = "1", features = ["derive"] }
```

CLI users install the `ezrs` binary:

```sh
cargo install ezrs
```

## CI And Release

The repository uses GitHub Actions for `cargo fmt --all --check`, `cargo check --workspace --all-targets`, `cargo test --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, and example compilation.

Publishing runs from the release workflow on GitHub release publication or manual dispatch. Maintainers must configure the repository secret `CARGO_REGISTRY_TOKEN` with a crates.io API token before running the publish workflow.

## Documentation Site

GitHub Pages publishes the documentation site at `https://christiandoxa.github.io/ezrs/`.

The Pages workflow builds Rust API docs with `cargo doc --workspace --no-deps`, renders README and guide pages, and includes source-rendered examples.

## Quickstart

```rust
use ezrs::prelude::*;

#[ezrs::main]
async fn main() -> Result<()> {
    App::new()
        .name("demo")
        .version("0.1.0")
        .command(hello)
        .run()
        .await
}

async fn hello(ctx: Context) -> Result<()> {
    ctx.println("hello from ezrs");
    Ok(())
}
```

Run:

```sh
cargo run -- hello
```

`App::command(hello)` derives the CLI command name from Rust syntax. A handler
named `hello` becomes `hello`, and `commands::scan::run` becomes `scan`.

## CLI Example

```rust
async fn scan(ctx: ezrs::Context) -> ezrs::Result<()> {
    let path = ctx.arg_or("path", ".");
    let recursive = ctx.flag("recursive");
    ctx.println(format!("scan path={path} recursive={recursive}"));
    Ok(())
}
```

```sh
cargo run -- scan --path src --recursive
```

## Worker Example

```rust
async fn work(ctx: ezrs::Context) -> ezrs::Result<()> {
    let worker_ctx = ctx.clone();
    ctx.spawn(async move {
        worker_ctx.println("background work");
        Ok(())
    });
    ctx.join_all().await
}
```

## Cancellation Example

Ctrl+C triggers cooperative cancellation during `App::run()`.

```rust
async fn watch(ctx: ezrs::Context) -> ezrs::Result<()> {
    loop {
        if ctx.is_cancelled() {
            ctx.println("shutting down");
            break;
        }
        ctx.check_cancelled()?;
        ctx.sleep_secs(1).await;
    }
    Ok(())
}
```

## State Without Arc

```rust
#[derive(Clone)]
struct State {
    app_name: String,
}

async fn hello(ctx: ezrs::Context) -> ezrs::Result<()> {
    let state = ctx.state::<State>()?;
    ctx.println(format!("app: {}", state.app_name));
    Ok(())
}
```

Users pass state through `App::state(State { ... })`. ezrs hides internal shared ownership.

## Shared And SharedMut

```rust
let greeting = ezrs::Shared::new(String::from("hello"));
assert_eq!(greeting.get(), "hello");
```

```rust
let counter = ezrs::SharedMut::new(0_u64);
counter.update(|n| *n += 1).await;
let n = *counter.read().await;
```

## Config Example

```rust
#[derive(Clone, serde::Deserialize)]
struct Config {
    workers: usize,
}

async fn run(ctx: ezrs::Context) -> ezrs::Result<()> {
    let cfg = ctx.config::<Config>()?;
    ctx.println(format!("workers: {}", cfg.workers));
    Ok(())
}
```

`App::config::<Config>()` loads `ezrs.toml` when present. `.env` is loaded for environment access through `ctx.env("KEY")`.

## Logging Example

```rust
async fn run(ctx: ezrs::Context) -> ezrs::Result<()> {
    ctx.log().info("started");
    ctx.log().warn("skipped optional file");
    ctx.log().error("failed example");
    Ok(())
}
```

Use `EZRS_LOG=debug` or `RUST_LOG=debug`.

## File Helper Example

```rust
async fn copy(ctx: ezrs::Context) -> ezrs::Result<()> {
    let text = ctx.fs().read_to_string("input.txt").await?;
    ctx.fs().write_string("out/output.txt", text).await?;
    for path in ctx.fs().walk(".")? {
        ctx.println(path.display());
    }
    Ok(())
}
```

## Process Example

```rust
async fn version(ctx: ezrs::Context) -> ezrs::Result<()> {
    let output = ctx.process("rustc")
        .arg("--version")
        .timeout_secs(5)
        .capture()
        .run()
        .await?;

    ctx.println(output.stdout_lossy().trim());
    Ok(())
}
```

This maps to Go's `exec.CommandContext`: explicit child process setup, timeout,
captured output, and status propagation.

## Persistence Example

```rust
async fn save(ctx: ezrs::Context) -> ezrs::Result<()> {
    let _lock = ctx.fs().try_lock("state.lock")?;
    ctx.fs().atomic_write_string("state.txt", "ready\n").await?;
    Ok(())
}
```

Use `read_json`, `write_json`, `read_toml`, and `write_toml` for typed local state.

## Testing Example

```rust
#[ezrs::test]
async fn hello_works() {
    let app = ezrs::App::new().command(hello);
    let res = app.test().args(["hello", "--name", "Ayu"]).run().await;
    res.assert_success();
    res.assert_stdout_contains("Ayu");
}
```

## Component Guide

Complete component examples live in `examples/components/`.

### App: building and running applications

Use `App::new()`, metadata builders, `command`, `default_command`, and `run`. See `examples/components/app.rs`.

### Context: args, flags, env, state, config, logging, fs, cancellation, tasks

`Context` is the app capability handle. It maps to `context.Context` plus app dependencies. See `examples/components/context.rs`.

### Result and Error

Use `ezrs::Result<()>`, `?`, and helpers like `Error::invalid_input`. See `examples/components/result_error.rs`.

### Dynamic args and flags

Use `ctx.arg("path")?`, `ctx.arg_or("path", ".")`, and `ctx.flag("recursive")`. See `examples/components/args_flags.rs`.

### State

Use `App::state(value)` and `ctx.state::<State>()?` for explicit dependency passing. See `examples/components/state.rs`.

### Shared and SharedMut

Use `Shared<T>` for read-only shared dependencies and `SharedMut<T>` for async mutable state. See `examples/components/shared.rs` and `examples/components/shared_mut.rs`.

### Config

Use `App::config::<T>()` and `ctx.config::<T>()?` with `serde::Deserialize`. See `examples/components/config.rs`.

### Logger

Use `ctx.log().info(...)`, `warn(...)`, and `error(...)`. See `examples/components/logger.rs`.

### Fs

Use `ctx.fs().read_to_string`, `write_string`, `exists`, `walk`, atomic writes, lock files, and typed JSON/TOML helpers. See `examples/components/fs.rs` and `examples/components/persistence.rs`.

### Task

Use `ctx.spawn(future)`, `ctx.join_all().await`, and `TaskGroup` for WaitGroup-style coordination. `spawn_named(...)` exists only as a low-level diagnostic escape hatch. See `examples/components/task_spawn.rs` and `examples/components/task_group.rs`.

### Cancellation

Use `ctx.is_cancelled()`, `ctx.cancelled().await`, and `ctx.check_cancelled()?`. See `examples/components/cancellation.rs`.

### Process

Use `ctx.process("program")` or `Process::new("program")` for `exec.CommandContext`-style child process execution. See `examples/components/process.rs`.

### Resilience

Use `RetryPolicy`, `retry`, `backoff_delay`, and `timeout` for retry/backoff and timeout loops. See `examples/components/resilience.rs`.

### Diagnostics

Use `DiagnosticRunner`, `Check`, and `DiagnosticReport` for doctor-style commands. See `examples/components/diagnostics.rs`.

### Reporting

Use `Report` and `Table` for plain CLI reports and simple JSON rendering. See `examples/components/reporting.rs`.

### Secrets

Use `SecretString` for redacted log-safe secret values with explicit exposure. See `examples/components/secrets.rs`.

### Test harness

Use `App::test()` for in-memory command tests. See `examples/components/test_harness.rs`.

### Macros

Use `#[ezrs::main]` and `#[ezrs::test]` for Tokio runtime setup. See `examples/components/macros_main.rs` and `examples/components/macros_test.rs`.

### CLI

Use `ezrs new`, `ezrs add command`, `ezrs run`, `ezrs check`, and `ezrs explain`. See `examples/components/cli_workflows.md`.

## For Go Developers

ezrs translates Go application patterns into Rust. It does not copy Go syntax.

- `func main() { if err := run(); err != nil { ... } }` maps to `#[ezrs::main] async fn main() -> ezrs::Result<()>`.
- `context.Context` maps to `ezrs::Context`.
- explicit app structs map to `#[derive(Clone)] State` passed through `App::state`.
- `if err != nil { return err }` maps to `?`.
- `sync.Mutex` app state maps to `SharedMut<T>`.
- goroutines map to `ctx.spawn`.
- WaitGroup-style coordination maps to `ctx.join_all`.
- `exec.CommandContext` maps to `ctx.process("program")`.
- temp-file rename and lock-file persistence map to `ctx.fs().atomic_write_string` and `ctx.fs().try_lock`.
- doctor commands map to `DiagnosticRunner`.
- redacted secrets map to `SecretString`.
- channels and select map to `tokio::sync::mpsc` and `tokio::select!`.
- table-driven tests map to normal Rust test loops plus `#[ezrs::test]`.

See `docs/golang-patterns.md` and `examples/golang_patterns/`.

## Go Tour Coverage

ezrs also includes a Go Tour mapping guide for developers learning Rust from Go concepts. See `docs/go-tour-mapping.md` and `examples/go_tour/`.

The guide covers the Go Tour topic families: basics, flow control, more types, methods and interfaces, generics, and concurrency. It maps each topic to idiomatic Rust and marks non-framework exercises as documented examples rather than new ezrs APIs.

## Golang Pattern Mapping Summary

ezrs directly supports app entrypoints, error-returning commands, context-style handlers, dynamic flags, config structs, env access, state, logging, file helpers, atomic persistence, process management, cancellation, task spawning, task groups, task joining, retry/backoff, diagnostics, reporting, redacted secrets, and in-memory command testing.

Idiomatic Rust examples cover traits as interfaces, RAII cleanup, channels, worker pools, pipelines, fan-out/fan-in, retry loops, tickers, table-driven tests, fake implementations, and service/repository layering.

## v0.1.0 Scope Boundaries

ezrs v0.1.0 is intentionally small. It targets CLI tools, workers, small daemons, file processors, automation tools, and background jobs.

Known limitations:

- typed argument derive is not implemented
- config environment merging is intentionally simple
- task cancellation is cooperative
- task panic capture is basic
- `ezrs explain` uses fixed pattern matching, not AI rewriting

## Non-Goals

ezrs v0.1.0 is not a web framework, ORM, microservice framework, distributed queue, OpenAPI framework, full scheduler, plugin system, GUI/TUI framework, Go-to-Rust translator, or automatic AI code rewrite tool.
