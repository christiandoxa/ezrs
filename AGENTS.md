# AGENTS.md

## Project

ezrs is a Rust 2024 application-pattern framework for simple CLI apps, workers, small daemons, file processors, automation tools, and background jobs.

## Product goal

ezrs exists so Go developers can apply familiar Golang application patterns in Rust.

It translates Go-style application architecture into idiomatic Rust APIs.

It must not copy Go syntax.

## Product philosophy

Keep ezrs boring but delightful.

Optimize for Go developers who want app-level Rust without ownership friction.

## v0.1.0 scope

Allowed:

- App builder
- Context
- simple Result/Error
- CLI command system
- dynamic args and flags
- state without user-facing Arc
- Shared and SharedMut
- simple config
- default logging
- file helpers
- task spawning
- Ctrl+C cancellation
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

## Golang pattern coverage

Future agents must preserve Golang pattern coverage.

This means translating Go patterns into idiomatic Rust and ezrs APIs.

When adding features, ask:

"Would this help a Go developer apply a familiar Go application pattern in Rust?"

If yes, implement it in a Rust-native way.

If no, keep it out.

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

Update component examples when component APIs change.

## Design principle

Prefer a small stable API over a large clever API.
