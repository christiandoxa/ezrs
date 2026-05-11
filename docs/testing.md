# Testing

ezrs test support is designed for Go-style table-driven tests while staying Rust-native.

## Owned Environment

Rust 2024 makes process environment mutation unsafe. Do not use `std::env::set_var` or `std::env::remove_var` in tests. Prefer injecting owned values with `EnvMap` or `TestEnv`.

```rust
use ezrs::test_support::EnvMap;

let env = EnvMap::new().set("PORT", "8080");
assert_eq!(env.get_string("PORT").as_deref(), Some("8080"));
```

Use `EnvMap::capture_current()` only when a test needs a snapshot of the real shell environment. After capture, pass the map as owned data.

## Temp Workspaces

`TempWorkspace` is a std-only RAII temporary directory. It creates a unique directory below `std::env::temp_dir()` and removes it on drop.

```rust
use ezrs::test_support::TempWorkspace;

let workspace = TempWorkspace::new("scan-test")?;
workspace.write("input.txt", "hello")?;
assert_eq!(workspace.read_to_string("input.txt")?, "hello");
# Ok::<(), ezrs::Error>(())
```

Use `Fixture` for compact file setup:

```rust
use ezrs::test_support::Fixture;

let fixture = Fixture::new("fixtures")?;
fixture.files([("a.txt", "a"), ("nested/b.txt", "b")])?;
# Ok::<(), ezrs::Error>(())
```

## Golden Assertions

`assert_golden(path, actual)` compares text against a golden file. Set `EZRS_ACCEPT_GOLDEN=1` in the shell to update files.

```rust
use ezrs::test_support::assert_golden;

assert_golden("tests/golden/report.txt", "report text\n");
```

The helper reads the environment but never mutates it.

## Fake Processes

Use `FakeProcessRunner`, `FakeCommandRequest`, and `FakeCommandOutput` for service tests that should not spawn real child processes.

```rust
use ezrs::test_support::{FakeCommandOutput, FakeCommandRequest, FakeProcessRunner};

let runner = FakeProcessRunner::new()
    .with_outputs([FakeCommandOutput::success("checked\n")]);

let output = runner.run(
    FakeCommandRequest::new("cargo").args(["check", "--workspace"]),
)?;

assert!(output.is_success());
assert_eq!(output.stdout_lossy(), "checked\n");
assert_eq!(runner.last_request().unwrap().program, "cargo");
# Ok::<(), ezrs::Error>(())
```

When process integration lands, real process code can convert fake requests into `crate::process` builders and fake output into process-compatible output types.

## Table-Driven Tests

Keep the Go pattern: define a local `Case` struct, list cases, loop, and include the case name in assertions.

```rust
struct Case {
    name: &'static str,
    input: &'static str,
    want: &'static str,
}

let cases = [
    Case { name: "empty", input: "", want: "default" },
    Case { name: "value", input: "Ayu", want: "hello Ayu" },
];

for case in cases {
    let got = greet(case.input);
    assert_eq!(got, case.want, "{}", case.name);
}
# fn greet(input: &str) -> String {
#     if input.is_empty() { String::from("default") } else { format!("hello {input}") }
# }
```
