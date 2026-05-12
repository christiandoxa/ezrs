# Typed Args

Typed args map dynamic CLI input into a Rust struct without adding a proc-macro crate.

This matches the Go `flag` package and cobra/pflag pattern: flags are still external user input, but handler code works with typed fields instead of repeating `ctx.arg("name")` and `ctx.flag("name")`.

Use `CommandSpec` and `ArgSpec` when the command should validate accepted
flags, support short aliases, render command-specific help, apply defaults, or
read env fallbacks. Use `TypedArgs` when handler code wants a Rust struct.
They compose cleanly: schema validates input first, then `TypedArgs` reads it.

```rust
let spec = ezrs::CommandSpec::new()
    .arg(ezrs::ArgSpec::option("path").short('p').required())
    .arg(ezrs::ArgSpec::flag("recursive").short('r'))
    .arg(ezrs::ArgSpec::option("limit").default("100").env("SCAN_LIMIT"));

ezrs::App::new().command_with(scan, spec).run().await
```

## Why There Is No Derive

Rust `#[derive(...)]` implementations must live in a proc-macro crate. `ezrs` intentionally publishes one crate for v0.1.0, so it cannot provide `#[derive(ezrs::Args)]` without restoring a second published crate.

The single-crate version uses ordinary traits and helper functions instead:

```rust
use ezrs::typed_args::{self, ArgSource, TypedArgs};
use ezrs::{Context, Result};

struct ScanArgs {
    path: String,
    recursive: bool,
    limit: usize,
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
        })
    }
}

async fn scan(ctx: Context) -> Result<()> {
    let args = ScanArgs::from_context(&ctx)?;

    ctx.println(format!("path={}", args.path));
    ctx.println(format!("recursive={}", args.recursive));
    ctx.println(format!("limit={}", args.limit));
    Ok(())
}
```

## API

- `TypedArgs`: implement this for a typed argument struct.
- `CommandSpec`: command-level schema for validation and help.
- `ArgSpec`: flag, option, or positional declaration.
- `FromArgs`: compatibility trait for building from `ezrs::Args`.
- `ArgSource`: common source trait implemented by `Args` and `Context`.
- `required(source, key)`: reads and parses a required named or positional argument.
- `optional(source, key)`: reads and parses an optional argument.
- `value_or(source, key, default)`: reads and parses an argument, or uses a typed default.
- `string_or(source, key, default)`: reads a string argument, or uses a default.
- `flag(source, key)`: reads a boolean flag.
- `positional(source, index)`: reads and parses a positional argument.
- `typed_args!`: optional macro for defining a struct and `TypedArgs` implementation in one place.

## Go Mapping

Go code often starts with package-level flag declarations:

```go
recursive := flag.Bool("recursive", false, "walk recursively")
path := flag.String("path", ".", "path to scan")
limit := flag.Int("limit", 100, "maximum files")
```

In ezrs, those names remain external CLI keys, but the command body receives normal Rust fields:

```rust
struct ScanArgs {
    path: String,
    recursive: bool,
    limit: usize,
}
```

This keeps Rust ownership and parsing explicit while removing repeated string lookups from handler logic.

## Macro Form

The `typed_args!` macro is available from this module once the module is exported by the crate root. It avoids a derive macro while keeping the field list compact:

```rust
ezrs::typed_args! {
    struct ScanArgs {
        path: String = |source| typed_args::string_or(source, "path", "."),
        recursive: bool = |source| typed_args::flag(source, "recursive"),
        limit: usize = |source| typed_args::value_or(source, "limit", 100)?,
    }
}
```

Each field names the source binding it wants to use. Use the helper functions with that binding.
