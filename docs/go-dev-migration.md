# Go Developer Migration Map

ezrs maps familiar Go application patterns into explicit Rust APIs. It does not
copy Go syntax and does not hide ownership or async behavior.

## CLI: cobra and pflag

Use Rust function items for command identity and `CommandSpec` for accepted
flags:

```rust
let spec = ezrs::CommandSpec::new()
    .arg(ezrs::ArgSpec::option("path").short('p').required())
    .arg(ezrs::ArgSpec::flag("recursive").short('r'))
    .arg(ezrs::ArgSpec::option("limit").default("100").env("SCAN_LIMIT"));

ezrs::App::new().command_with(scan, spec).run().await
```

Use `TypedArgs` when handler code should work with a Rust struct after schema
validation.

## context.Context

Use `Context` for cancellation, deadlines, args, env, state, config, logging,
filesystem helpers, child processes, tasks, and captured test output.

`ctx.process(...)`, `ctx.err_group()`, and `retry_with_cancellation(...)` are
cancellation-aware building blocks.

## errgroup.WithContext

Use `ctx.err_group()` or `TaskGroup::err_group()`:

```rust
let group = ctx.err_group();
group.spawn_named_with_cancel("worker", |cancellation| async move {
    cancellation.check_cancelled()?;
    Ok(())
});
group.join().await?;
```

## exec.CommandContext

Use `ctx.process("program")` for Context-bound child processes. Use
`Process::new("program")` only when you need the low-level detached escape
hatch.

## Config and env

Use `ConfigSource` for layered config:

```rust
App::new().config_validated::<Config, _>(
    ezrs::ConfigSource::ezrs().env_prefix("APP").required(),
    |config| {
        if config.workers == 0 {
            Err(ezrs::Error::invalid_input("workers must be greater than zero"))
        } else {
            Ok(())
        }
    },
)
```

`APP_WORKERS=8` overrides the `workers` config key. Use double underscores for
nested keys, such as `APP_DATABASE__URL`.

## Logging

Use simple messages for beginner code and field-style logs when stable keys
matter:

```rust
ctx.log().info("started");
ctx.log().info_fields("worker ready", [("worker", "scan")]);
```

## Tests

Use `App::test()` for command handlers, `EnvMap` for owned test environment,
`TempWorkspace` for isolated files, `FakeProcessRunner` for service tests, and
`FakeClock` for time-dependent logic.

## CLI workflow

Use `ezrs check` before pushing. It runs formatting, check, tests, clippy with
warnings denied, and example compilation. Use `ezrs explain --last-error` to
explain the last captured failure in Go-developer terms.
