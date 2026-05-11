//! ezrs facade crate.
//!
//! Most users should depend only on this crate.

pub use ezrs_core::{App, Context};
pub use ezrs_error::{Error, Result};
pub use ezrs_macros::{main, test};
pub use ezrs_shared::{Shared, SharedMut};

pub mod prelude;

#[doc(hidden)]
pub mod __private {
    pub use tokio;
}
