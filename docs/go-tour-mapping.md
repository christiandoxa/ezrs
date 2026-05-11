# Go Tour Mapping

ezrs does not try to reproduce Go syntax. This guide maps the official Go Tour topic families to idiomatic Rust and, where relevant, ezrs application APIs.

`examples/go_tour/` contains compiling Rust examples for each practical topic family. These examples are teaching examples; they are not a Go compatibility layer.

## Matrix

| Go Tour topic | Rust equivalent | ezrs relevance | Example |
|---|---|---|---|
| Packages | crates and modules | `ezrs` is one published crate | `examples/go_tour/basics_packages_imports.rs` |
| Imports | `use` paths | examples import `ezrs::{App, Context, Result}` | `examples/go_tour/basics_packages_imports.rs` |
| Exported names | `pub`, not capitalization | public API is explicit | `examples/go_tour/basics_packages_imports.rs` |
| Functions | `fn` and `async fn` | commands are async functions | `examples/go_tour/basics_functions_results.rs` |
| Multiple results | tuples | command results use `Result<()>` | `examples/go_tour/basics_functions_results.rs` |
| Named return values | explicit local bindings and final expression | not copied; Rust favors explicit returns | `examples/go_tour/basics_functions_results.rs` |
| Variables | `let` and `let mut` | app state is explicit | `examples/go_tour/basics_variables_types_constants.rs` |
| Basic types | Rust scalar types | config/state fields use Rust types | `examples/go_tour/basics_variables_types_constants.rs` |
| Zero values | `Default` or explicit initialization | no implicit zero-value model | `examples/go_tour/basics_variables_types_constants.rs` |
| Type conversion | `as`, `From`, `TryFrom`, parse methods | use explicit conversion in app code | documented here |
| Type inference | Rust infers local types | public API keeps types clear | `examples/go_tour/basics_variables_types_constants.rs` |
| Constants | `const NAME: Type` | use for app defaults | `examples/go_tour/basics_variables_types_constants.rs` |
| For | `for`, `while`, `loop` | worker loops check cancellation | `examples/go_tour/basics_flow_control.rs` |
| If | `if` expression | command validation | `examples/go_tour/basics_flow_control.rs` |
| Switch | `match` | command branching and state handling | `examples/go_tour/basics_flow_control.rs` |
| Defer | RAII and `Drop` | cleanup is ownership-driven | `examples/go_tour/basics_flow_control.rs` |
| Pointers | references and smart pointers | users normally avoid `Arc` via `Shared` | `examples/go_tour/more_types_pointers_structs.rs` |
| Structs | `struct` | app state and config | `examples/go_tour/more_types_pointers_structs.rs` |
| Arrays | `[T; N]` | regular Rust data model | `examples/go_tour/more_types_arrays_slices.rs` |
| Slices | `&[T]` and `Vec<T>` | command helpers can process slices | `examples/go_tour/more_types_arrays_slices.rs` |
| Maps | `HashMap<K, V>` | use with `SharedMut` for cache state | `examples/go_tour/more_types_maps.rs` |
| Range | iterators and ranges | file walking and worker loops | `examples/go_tour/more_types_range.rs` |
| Function values | function pointers and closures | handlers are function values registered in `App` | `examples/go_tour/more_types_functions_closures.rs` |
| Closures | closures with borrow or `move` capture | spawned tasks often use `async move` | `examples/go_tour/more_types_functions_closures.rs` |
| Methods | `impl Type` blocks | services and domain structs use methods | `examples/go_tour/methods_interfaces_methods.rs` |
| Pointer receivers | `&mut self` | mutation is explicit | `examples/go_tour/methods_interfaces_methods.rs` |
| Interfaces | traits | Go interfaces map to Rust traits | `examples/go_tour/methods_interfaces_traits.rs` |
| Interface values | trait objects or generics | prefer generics for static dispatch | `examples/go_tour/methods_interfaces_traits.rs` |
| Empty interface | `dyn Any` or generics | not a common public ezrs pattern | documented here |
| Type assertions | downcasting or enums | `ctx.state::<T>()?` uses typed lookup | `examples/go_tour/methods_interfaces_traits.rs` |
| Type switches | `match` on enums or `Any` downcast | prefer enums | documented here |
| Stringers | `Display` trait | log/output accepts displayable values | `examples/go_tour/generics_constraints.rs` |
| Errors | `Result<T, E>` and `?` | `ezrs::Error` and `ezrs::Result` | `examples/go_tour/methods_interfaces_errors.rs` |
| Readers | `std::io::Read`, Tokio async read traits | `ctx.read_stdin().await?` and fs helpers | `examples/go_tour/methods_interfaces_readers.rs` |
| Images | image packages | out of ezrs scope | out of scope |
| Type parameters | Rust generics | useful for services and repositories | `examples/go_tour/generics_type_parameters.rs` |
| Generic constraints | trait bounds | maps directly to Rust trait bounds | `examples/go_tour/generics_constraints.rs` |
| Goroutines | Tokio tasks | `ctx.spawn` | `examples/go_tour/concurrency_tasks.rs` |
| Channels | `tokio::sync::mpsc` | used directly in examples | `examples/go_tour/concurrency_channels.rs` |
| Buffered channels | `mpsc::channel(capacity)` | used directly in examples | `examples/go_tour/concurrency_channels.rs` |
| Range and close over channels | `while let Some(v) = rx.recv().await`; close by dropping sender | used directly in examples | `examples/go_tour/concurrency_channels.rs` |
| Select | `tokio::select!` | combines channels, timers, cancellation | `examples/go_tour/concurrency_select.rs` |
| Default select | `else` or timeout branch depending on pattern | use Tokio patterns | `examples/go_tour/concurrency_select.rs` |
| Mutex | `SharedMut<T>` or `tokio::sync::Mutex` | `SharedMut<T>` avoids user-facing lock plumbing | `examples/go_tour/concurrency_mutex.rs` |
| Equivalent binary trees exercise | recursion and channels | teaching exercise, not framework API | documented here |
| Web crawler exercise | async tasks, channels, shared state | app-level pattern, not HTTP framework | documented here |

## Out Of Scope

The Go Tour includes language exercises that are not application framework APIs. ezrs documents their Rust equivalents but does not add framework features for them.

Out of scope for ezrs v0.1.0:

- copying Go syntax
- Go package/runtime compatibility
- image processing helpers
- HTTP crawler framework
- automatic solution generation for Go Tour exercises
- broad code generation beyond `#[ezrs::main]` and `#[ezrs::test]`

## How This Relates To Existing ezrs Examples

- `examples/components/` teaches each ezrs component directly.
- `examples/golang_patterns/` teaches Go application architecture patterns.
- `examples/go_tour/` maps the Go Tour language and concurrency lessons to Rust and ezrs.
