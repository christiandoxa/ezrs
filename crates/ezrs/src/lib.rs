//! ezrs - Go-style application patterns, Rust-grade safety.
//!
//! This package contains the public library and the `ezrs` CLI binary.

mod app;
mod args;
mod command;
mod config;
mod context;
mod diagnostic;
mod error;
mod fs;
mod log;
mod process;
mod report;
mod resilience;
mod secret;
mod shared;
mod state;
mod task;

pub use app::{App, AppTest, CommandGroup, TestResult};
pub use args::Args;
pub use context::Context;
pub use diagnostic::{Check, CheckStatus, DiagnosticReport, DiagnosticRunner};
pub use error::{Error, Result};
pub use fs::{FileLock, Fs};
pub use process::{Process, ProcessOutput, ProcessStatus};
pub use report::{Report, Table};
pub use resilience::{RetryPolicy, backoff_delay, retry, timeout};
pub use secret::SecretString;
pub use shared::{Shared, SharedMut};
pub use task::{Cancellation, TaskGroup};
pub use tokio::{main, test};

pub mod prelude;

/// Builds a nested command group from Rust handler syntax.
#[macro_export]
macro_rules! command_group {
    ($group:ident { $($command:path),+ $(,)? }) => {{
        let group = $crate::CommandGroup::__from_static(stringify!($group));
        $(
            let group = group.command($command);
        )+
        group
    }};
}

#[doc(hidden)]
pub mod __private {
    pub use tokio;
}
