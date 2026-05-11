# AGENTS.md

## Project

ezrs is a Rust 2024 general-purpose application framework for CLI apps, workers, small daemons, file processors, automation tools, background jobs, and local app-runtime systems.

The first public angle is Go-pattern-to-Rust application development, but the long-term direction is broader: provide boring, composable Rust building blocks for everyday application infrastructure.

## Product goal

ezrs exists so developers can build practical Rust applications with less repeated infrastructure code.

For Go developers, ezrs translates Go-style application architecture into idiomatic Rust APIs.

For Rust developers generally, ezrs should provide small, composable building blocks for application entrypoints, command routing, state, config, logging, tasks, cancellation, process management, persistence, observability, testing, and runtime operations.

It must not copy Go syntax.

It must not become a magic framework that hides Rust.

It should make Rust excellent for boring, real application work.

## Product philosophy

Keep ezrs boring but delightful.

Optimize for Go developers who want app-level Rust without ownership friction, while keeping the framework useful to Rust developers who simply want reliable application building blocks.

Prefer primitives that compose with the Rust ecosystem over closed framework abstractions.

Prefer syntax-first APIs over stringly typed APIs.

When Rust syntax can express identity, ownership, type, or behavior directly, use that syntax instead of placeholder strings like `"bin"`, `"name"`, `"worker"`, or `"command"`.

Strings are acceptable for external data and user input: CLI arg keys, environment variable names, filesystem paths, process binary paths, HTTP header names, metric labels, and user-facing text.

Strings should not be the primary way to model framework structure when a function item, type, enum variant, const, trait implementation, builder method, or macro can express the structure more safely.

## v0.1.0 scope

Allowed:

- App builder
- Context
- simple Result/Error
- CLI command system
- dynamic args and flags
- typed args without a proc-macro crate
- state without user-facing Arc
- Shared and SharedMut
- simple config
- default logging
- file helpers
- atomic local persistence
- child process management
- lifecycle hooks
- task spawning
- task groups
- channel/select helpers
- worker pools
- Ctrl+C cancellation
- retry/backoff and timeout helpers
- doctor-style diagnostics
- plain CLI reports
- redacted secrets
- test harness
- ezrs new/add/run/check/explain
- Golang pattern documentation and examples
- component usage examples

Not allowed:

- web framework
- ORM
- plugin system
- distributed queue
- OpenAPI
- full scheduler
- GUI/TUI framework
- Go-to-Rust translator
- automatic AI rewrite
- microservice framework

These are v0.1.0 limits, not permanent product limits. Future versions may add optional integrations or app-runtime helpers when they are framed as reusable application building blocks, not as broad replacement frameworks.

## Framework building blocks

Future ezrs work should be evaluated as framework building blocks. A block does not have to be implemented fully inside ezrs. It may be:

- core ezrs API
- optional ezrs adapter
- documented external crate integration
- example-only pattern
- explicit out-of-scope decision

The desired building block map is:

- App entrypoint and runtime: `#[ezrs::main]`, Tokio setup, shutdown orchestration, app metadata, lifecycle hooks.
- Command system: command registration, command aliases, hidden commands, default command, nested commands, passthrough args, typed args, help/version output, shell-friendly errors.
- Context and capabilities: cheap clone handle for args, env, state, config, logging, fs, cancellation, tasks, output, and future app capabilities.
- Error model: `Result`, human-readable errors, exit codes, error categories, context wrapping, diagnostic-friendly messages.
- State and dependencies: explicit state passing, typed lookup, immutable shared state, mutable shared state, fakeable services, trait-backed repositories.
- Config and environment: config files, `.env`, env access, typed config, simple layering, validation, secrets separation.
- Logging and tracing: default logger, structured fields, spans, request/task correlation, file logs where needed.
- Filesystem and persistence: read/write helpers, walk helpers, atomic writes, file locks, backup recovery, JSON/TOML helpers, append-only journals.
- Task and concurrency: named tasks, joins, cancellation, channel/select helpers, worker pools, fan-out/fan-in, bounded queues, timeout helpers, select-style examples.
- Process management: child process spawning, env overlays, stdin/stdout/stderr wiring, exit status propagation, kill-on-drop, PID/lease helpers. Use direct Rust syntax for process specs where possible; avoid placeholder-first APIs.
- Networking integration: HTTP client examples, local admin HTTP helpers, streaming/SSE/WebSocket examples or adapters, proxy patterns where needed. This must not turn ezrs into a general web framework by accident.
- Runtime operations: health checks, doctor commands, readiness/liveness reports, runtime registries, local daemon/broker lifecycle helpers.
- Resilience: retry, backoff, circuit breaker, admission control, rate limiting, overload handling, graceful degradation.
- Observability: metrics snapshots, Prometheus text rendering, audit events, structured diagnostic reports, redaction-safe output.
- Security and secrets: redaction helpers, secret locations, token loading, secret-store abstraction, no secret leakage in logs/tests.
- Testing: command harness, fake dependencies, temp homes/workspaces, env isolation, fixture loading, golden assertions, fake process runners, replay tests for JSON/SSE/WebSocket-style protocols.
- Terminal output: plain text reports, tables, panels, optional TUI integration through external crates.
- Packaging and release: project scaffolding, GitHub Actions, Pages docs, publish workflow, version bump guidance.
- Documentation and examples: component examples, Go pattern examples, Go Tour mappings, cookbook-style advanced app examples.

When deciding whether a building block belongs in ezrs, ask:

"Is this reusable application infrastructure that many Rust apps need?"

"Can it stay small, explicit, testable, and composable?"

"Should ezrs implement it, wrap an ecosystem crate, or only document the integration?"

If the answer is unclear, document the pattern first before adding public API.

## Core vs adapters

Keep the core crate small and stable.

Good core candidates:

- app lifecycle
- context capabilities
- command routing
- errors
- state
- config
- logging
- fs helpers
- process management
- local persistence helpers
- diagnostics and reporting
- secret redaction
- task/cancellation
- command tests

Good optional adapter candidates:

- typed CLI via clap
- derive-based CLI arguments if ezrs restores a separate proc-macro crate
- HTTP client/server helpers
- WebSocket/SSE helpers
- metrics exporters
- terminal UI rendering
- secret backends
- advanced persistence stores
- advanced process supervision

Good external-only candidates:

- ORM
- full web framework
- distributed queue
- full scheduler
- OpenAPI framework
- GUI/TUI framework
- actor framework

ezrs may document or integrate with those ecosystems, but should not replace them unless there is a clear app-pattern reason.

## Syntax-first API rule

Future public APIs should avoid stringly typed framework structure.

Preferred:

- `App::new().command(scan)` when command name can be derived from the function item.
- `App::new().commands(commands![scan, inspect, repair])` when a macro can preserve readable syntax.
- `ctx.spawn(worker())` or `ctx.tasks().spawn(worker)` when a function item or task spec carries identity.
- `ctx.process(cargo()).arg("check").run().await?` when a typed process spec can be expressed directly.
- `#[derive(ezrs::Args)] struct ScanArgs { path: PathBuf, recursive: bool }` for typed CLI input.
- `TypedArgs` or `typed_args!` while ezrs remains a single published crate and cannot host derive macros.
- enum-based command trees for nested commands when using a typed CLI adapter.

Acceptable:

- `ctx.arg("path")` for dynamic CLI compatibility.
- `ctx.env("HOME")` because environment variables are external strings.
- `ctx.fs().read_to_string("file.txt")` because paths are external data.
- `ctx.log().info("started")` because messages are text.
- `ctx.process("cargo")` as a low-level escape hatch, not the preferred high-level pattern.

Avoid for new high-level APIs:

- `ctx.command("bin")` where `"bin"` is a placeholder for structure.
- `ctx.worker_pool("name", ...)` when the worker function/type can define identity.
- `ctx.metric("name", ...)` without a typed metric descriptor for stable metrics.
- `App::command("scan", scan)` as the only path for typed command systems.

Compatibility note:

- Existing v0.1.0 dynamic APIs may remain for simple scripts and Go-style low ceremony.
- Newer ergonomic APIs should offer syntax-first alternatives.
- Do not break beginner examples without providing a clearer syntax-first replacement.
- While ezrs publishes one crate, prefer trait-plus-macro APIs over derive macros that require a proc-macro crate.

## Prodex-class application support

Prodex-like apps are a useful stress test for ezrs.

A Prodex rewrite should be able to use ezrs for the application shell, command structure, state, config, logging, tasks, cancellation, tests, docs, and some filesystem helpers.

Prodex-class gaps that ezrs may eventually support as reusable blocks:

- nested typed command trees and passthrough args
- child process orchestration
- atomic persistence and merge-safe file state
- local runtime broker lifecycle
- health and doctor reports
- structured metrics and audit logs
- redaction and secret handling
- retry/backoff/circuit/admission primitives
- streaming protocol test/replay helpers

Prodex-specific domain logic must remain outside ezrs.

## Language rules

All code, comments, docs, examples, commit-style notes, and CLI text must be written in English.

## Rust rules

- Use Rust 2024 edition.
- Prefer stable Rust.
- Avoid nightly-only features.
- Prefer simple, readable APIs.
- Do not expose Arc or Mutex in common user-facing examples.
- Make examples copy-paste compile where practical.

## Architecture rules

This repository publishes one crate: ezrs.

The crate contains the public library and the `ezrs` CLI binary.

Keep internal modules focused:

- app and context own App, Context, command routing, output capture, and state.
- error owns Error and Result.
- shared owns Shared and SharedMut.
- config owns config and .env loading.
- log owns default logging.
- fs owns file helpers.
- task owns task spawning and cancellation.
- bin/ezrs owns the ezrs command line tool.

Do not leak internal module complexity into beginner examples.

Do not add new crates unless publishing multiple crates is intentionally restored.

Internal modules may grow as building blocks mature, but public API stability matters more than internal module purity.

## Golang pattern coverage

Future agents must preserve Golang pattern coverage.

This means translating Go patterns into idiomatic Rust and ezrs APIs.

When adding Go-oriented features, ask:

"Would this help a Go developer apply a familiar Go application pattern in Rust?"

If yes, implement it in a Rust-native way.

If no, evaluate it under the general building-block criteria instead of rejecting it only because it is not Go-specific.

## Component examples

Future agents must preserve examples/components/.

Every major public component must have a small, practical usage example.

Component examples must use the public facade crate ezrs where practical.

## Commands

Before finishing any change, run:

```sh
cargo fmt --all
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

If clippy is too strict during early scaffolding, document the reason clearly and still keep warnings low.

## Testing expectations

Add tests for:

- command routing
- args and flags parsing
- state lookup
- config loading
- SharedMut updates
- cancellation checks where practical
- test harness assertions
- ezrs CLI project generation
- key Golang pattern examples
- component examples where practical

## Documentation expectations

Update README.md when public API changes.

Update docs/golang-patterns.md when adding or changing Go-pattern-related behavior.

Update docs/go-tour-mapping.md and examples/go_tour/ when changing Go Tour coverage.

Update component examples when component APIs change.

## Design principle

Prefer a small stable API over a large clever API.
