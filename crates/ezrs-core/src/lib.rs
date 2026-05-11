//! Core ezrs app model: App, Context, command routing, dynamic args, and state.

mod app;
mod args;
mod command;
mod context;
mod state;

pub use app::{App, AppTest, TestResult};
pub use args::Args;
pub use context::Context;
