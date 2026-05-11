# Golang Patterns In ezrs

ezrs helps Go developers apply familiar application architecture in Rust. It does not copy Go syntax, hide Rust, or emulate the Go standard library. The goal is translation: recognizable Go patterns implemented as idiomatic Rust APIs, traits, ownership, async tasks, and tests.

Use `examples/golang_patterns/` for pattern-focused examples. Use `examples/components/` for direct ezrs component examples.

## Compatibility Matrix

| # | Go pattern | Rust or ezrs equivalent | Coverage |
|---|---|---|---|
| 1 | Application entrypoint | `#[ezrs::main] async fn main() -> ezrs::Result<()>` | `examples/golang_patterns/app_entrypoint.rs` |
| 2 | `run() error` | `async fn run(ctx: Context) -> Result<()>` | direct API |
| 3 | `context.Context` | `ezrs::Context` | `examples/components/context.rs` |
| 4 | Explicit dependency passing | `#[derive(Clone)] State` plus `App::state` | `examples/components/state.rs` |
| 5 | Shared app state | `SharedMut<HashMap<_, _>>` | `examples/golang_patterns/shared_state.rs` |
| 6 | Constructor pattern | `impl Service { pub fn new(...) -> Self }` | documented Rust pattern |
| 7 | Options pattern | Rust builder methods like `App::new().name(...).version(...)` | `examples/golang_patterns/builder_options.rs` |
| 8 | Interface-based design | Rust traits | `examples/golang_patterns/traits_interfaces.rs` |
| 9 | Small interface | `std::io::Read`, async traits, or small custom traits | documented Rust pattern |
| 10 | Composition over inheritance | structs containing dependencies or generic fields | documented Rust pattern |
| 11 | Error as value | `Result<T>` and `?` | `examples/components/result_error.rs` |
| 12 | Custom error | `Error::not_found`, `Error::invalid_input`, or app enums | direct API |
| 13 | Sentinel error | helper constructors like `Error::not_found(...)` | direct API |
| 14 | Wrapping error | `Error::msg(format!("load config: {err}"))` | direct API |
| 15 | `defer` cleanup | RAII and `Drop` | `examples/golang_patterns/cleanup_raii.rs` |
| 16 | Goroutine | `ctx.spawn("worker", async move { ... })` | `examples/golang_patterns/tasks_goroutines.rs` |
| 17 | Worker pool | `tokio::sync::mpsc` plus `ctx.spawn` | `examples/golang_patterns/worker_pool.rs` |
| 18 | Channel communication | `tokio::sync::mpsc` | `examples/golang_patterns/channels.rs` |
| 19 | Buffered channel | `mpsc::channel(capacity)` | documented Rust pattern |
| 20 | Channel close | drop all senders | documented Rust pattern |
| 21 | Range over channel | `while let Some(v) = rx.recv().await` | `examples/golang_patterns/channels.rs` |
| 22 | `select` | `tokio::select!` | `examples/golang_patterns/select_cancellation.rs` |
| 23 | Context cancellation | `ctx.cancelled().await` and `ctx.check_cancelled()?` | direct API |
| 24 | Graceful shutdown | Ctrl+C cancels Context during `App::run()` | `examples/components/cancellation.rs` |
| 25 | Timeout | `tokio::time::timeout(...)` | documented Rust pattern |
| 26 | Ticker | `tokio::time::interval(...)` | `examples/golang_patterns/ticker.rs` |
| 27 | Mutex | `SharedMut<T>` | direct API |
| 28 | Once initialization | `std::sync::OnceLock` or Tokio once cells | documented Rust pattern |
| 29 | WaitGroup | `ctx.spawn` plus `ctx.join_all().await` | direct API |
| 30 | Pipeline | mpsc stages | `examples/golang_patterns/pipeline.rs` |
| 31 | Fan-out fan-in | mpsc jobs plus results channel | `examples/golang_patterns/fan_out_fan_in.rs` |
| 32 | Rate limiting | interval or `tokio::sync::Semaphore` | documented Rust pattern |
| 33 | Retry with backoff | loop over `Result` plus sleep | `examples/golang_patterns/retry_backoff.rs` |
| 34 | CLI command | `App::command("scan", scan)` | direct API |
| 35 | Flags | `ctx.arg_or("path", ".")` and `ctx.flag("recursive")` | `examples/components/args_flags.rs` |
| 36 | Config struct | `serde::Deserialize` plus `App::config::<T>()` | direct API |
| 37 | Environment config | `ctx.env("PORT")?` | direct API |
| 38 | Logging | `ctx.log().info(...)` | `examples/components/logger.rs` |
| 39 | Table-driven tests | Rust case structs and loops | `examples/golang_patterns/table_driven_tests.rs` |
| 40 | Handler testing | `App::test().args([...]).run().await` | direct API |
| 41 | Fake implementation | fake structs implementing traits | `examples/golang_patterns/fake_implementations.rs` |
| 42 | Package layout | workspace crates and modules | repository layout |
| 43 | internal package | non-public modules and non-reexported crates | repository layout |
| 44 | Standard file helpers | `ctx.fs().read_to_string`, `write_string`, `walk` | direct API |
| 45 | HTTP/service pattern | service layer maps; HTTP framework out of scope | out of scope for v0.1.0 |
| 46 | Middleware | function composition; middleware framework out of scope | documented only |
| 47 | Repository/service layering | command -> service -> trait-backed repository | `examples/golang_patterns/service_repository.rs` |
| 48 | DI without framework | manual state construction in main | direct API |
| 49 | Build tags/features | Cargo features and `cfg` | documented Rust pattern |
| 50 | Code generation | `build.rs`, proc macros; ezrs only ships main/test macros | limited scope |
| 51 | Formatting/tooling | `cargo fmt`, `cargo test`, `cargo clippy`; `ezrs check` | CLI |
| 52 | Module/dependency | Cargo.toml and Cargo.lock | documented Rust pattern |
| 53 | Simple binary scaffold | `ezrs new myapp` | CLI |
| 54 | Add command | `ezrs add command scan` | CLI |
| 55 | Explain compiler error | `ezrs explain --last-error` fixed advice | CLI |

## Notes For Go Developers

### Context

Go `context.Context` usually carries cancellation and request-scoped values. ezrs `Context` carries cancellation plus app capabilities: args, env, state, config, logging, fs helpers, task spawning, output, and test capture.

### Dependencies

Prefer explicit state:

```rust
#[derive(Clone)]
struct State {
    service: Service,
}
```

Then pass it with `App::state(state)` and retrieve it with `ctx.state::<State>()?`.

### Errors

Use `ezrs::Result<T>` and `?`. This is the Rust equivalent of checking `if err != nil` after every call.

### Channels And Select

ezrs does not wrap Tokio channels in v0.1.0. Use `tokio::sync::mpsc` directly for channel patterns and `tokio::select!` for select-style coordination.

### Testing

Use `App::test()` when testing commands. Use normal Rust loops for table-driven tests.

## Out Of Scope In v0.1.0

ezrs v0.1.0 does not implement a web framework, ORM, full scheduler, middleware framework, distributed queue, plugin system, GUI/TUI framework, Go-to-Rust translator, or automatic AI code rewrite system.

HTTP and middleware patterns can still be modeled as services and function composition, but ezrs does not own HTTP routing or middleware layers in v0.1.0.
