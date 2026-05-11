//! ezrs - Go-style application patterns, Rust-grade safety.
//!
//! This package contains the public library and the `ezrs` CLI binary.

mod app;
mod args;
mod command;
mod config;
mod context;
mod error;
mod fs;
mod log;
mod shared;
mod state;
mod task;

pub use app::{App, AppTest, TestResult};
pub use args::Args;
pub use context::Context;
pub use error::{Error, Result};
pub use shared::{Shared, SharedMut};
pub use tokio::{main, test};

pub mod prelude;

#[doc(hidden)]
pub mod __private {
    pub use tokio;
}
