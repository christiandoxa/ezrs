//! Typed argument helpers for Go flag/cobra-style command input.

use std::fmt::Display;
use std::str::FromStr;

use crate::{Args, Context, Error, Result};

/// Reads dynamic command arguments from a common source.
///
/// Implemented for [`Args`] and [`Context`] so typed argument structs can be
/// tested against parsed argv data and used directly inside command handlers.
pub trait ArgSource {
    /// Returns a named or positional argument value.
    fn value(&self, key: &str) -> Option<String>;

    /// Returns true when a flag was present.
    fn flag(&self, key: &str) -> bool;
}

impl ArgSource for Args {
    fn value(&self, key: &str) -> Option<String> {
        self.get(key).map(ToOwned::to_owned)
    }

    fn flag(&self, key: &str) -> bool {
        self.flag(key)
    }
}

impl ArgSource for Context {
    fn value(&self, key: &str) -> Option<String> {
        self.arg(key).ok()
    }

    fn flag(&self, key: &str) -> bool {
        self.flag(key)
    }
}

/// Builds a typed argument struct from dynamic command arguments.
pub trait TypedArgs: Sized {
    /// Builds `Self` from any supported argument source.
    fn from_source<S>(source: &S) -> Result<Self>
    where
        S: ArgSource + ?Sized;

    /// Builds `Self` from a command [`Context`].
    fn from_context(ctx: &Context) -> Result<Self> {
        Self::from_source(ctx)
    }
}

/// Compatibility alias for users who prefer the `FromArgs` name.
pub trait FromArgs: Sized {
    /// Builds `Self` from parsed [`Args`].
    fn from_args(args: &Args) -> Result<Self>;
}

impl<T> FromArgs for T
where
    T: TypedArgs,
{
    fn from_args(args: &Args) -> Result<Self> {
        T::from_source(args)
    }
}

/// Reads and parses a required named or positional argument.
pub fn required<S, T>(source: &S, key: &str) -> Result<T>
where
    S: ArgSource + ?Sized,
    T: FromStr,
    T::Err: Display,
{
    let value = source
        .value(key)
        .ok_or_else(|| Error::not_found(format!("argument '{key}'")))?;

    parse_value(key, &value)
}

/// Reads and parses an optional named or positional argument.
pub fn optional<S, T>(source: &S, key: &str) -> Result<Option<T>>
where
    S: ArgSource + ?Sized,
    T: FromStr,
    T::Err: Display,
{
    source
        .value(key)
        .map(|value| parse_value(key, &value))
        .transpose()
}

/// Reads and parses an argument, or returns `default` when absent.
pub fn value_or<S, T>(source: &S, key: &str, default: T) -> Result<T>
where
    S: ArgSource + ?Sized,
    T: FromStr,
    T::Err: Display,
{
    optional(source, key).map(|value| value.unwrap_or(default))
}

/// Reads a string argument, or returns `default` when absent.
pub fn string_or<S>(source: &S, key: &str, default: impl Into<String>) -> String
where
    S: ArgSource + ?Sized,
{
    source.value(key).unwrap_or_else(|| default.into())
}

/// Reads a boolean flag.
pub fn flag<S>(source: &S, key: &str) -> bool
where
    S: ArgSource + ?Sized,
{
    source.flag(key)
}

/// Reads and parses a required positional argument by index.
pub fn positional<S, T>(source: &S, index: usize) -> Result<T>
where
    S: ArgSource + ?Sized,
    T: FromStr,
    T::Err: Display,
{
    required(source, &index.to_string())
}

fn parse_value<T>(key: &str, value: &str) -> Result<T>
where
    T: FromStr,
    T::Err: Display,
{
    value
        .parse()
        .map_err(|err| Error::invalid_input(format!("argument '{key}' value '{value}': {err}")))
}

/// Defines a typed argument struct and its [`TypedArgs`] implementation.
///
/// Each right-hand side expression names the argument source it reads from.
#[macro_export]
macro_rules! typed_args {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field:ident : $ty:ty = |$source:ident| $reader:expr
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name {
            $(
                $(#[$field_meta])*
                $field_vis $field: $ty,
            )*
        }

        impl $crate::typed_args::TypedArgs for $name {
            fn from_source<S>(source: &S) -> $crate::Result<Self>
            where
                S: $crate::typed_args::ArgSource + ?Sized,
            {
                Ok(Self {
                    $(
                        $field: {
                            let $source = source;
                            $reader
                        },
                    )*
                })
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct ScanArgs {
        path: String,
        recursive: bool,
        limit: usize,
        first: Option<String>,
    }

    impl TypedArgs for ScanArgs {
        fn from_source<S>(source: &S) -> Result<Self>
        where
            S: ArgSource + ?Sized,
        {
            Ok(Self {
                path: string_or(source, "path", "."),
                recursive: flag(source, "recursive"),
                limit: value_or(source, "limit", 10)?,
                first: optional(source, "0")?,
            })
        }
    }

    fn parse(args: &[&str]) -> Args {
        Args::parse(
            &args
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>(),
        )
    }

    #[test]
    fn builds_typed_args_from_dynamic_args() {
        let args = parse(&["input.txt", "--path", "src", "--recursive", "--limit=20"]);

        let typed = ScanArgs::from_args(&args).expect("typed args");

        assert_eq!(
            typed,
            ScanArgs {
                path: "src".to_string(),
                recursive: true,
                limit: 20,
                first: Some("input.txt".to_string()),
            }
        );
    }

    #[test]
    fn reports_parse_errors_with_argument_name() {
        let args = parse(&["--limit", "many"]);

        let err = ScanArgs::from_args(&args).expect_err("parse should fail");

        assert!(err.to_string().contains("argument 'limit' value 'many'"));
    }

    typed_args! {
        #[derive(Debug, PartialEq, Eq)]
        struct MacroArgs {
            path: String = |source| string_or(source, "path", "."),
            recursive: bool = |source| flag(source, "recursive"),
        }
    }

    #[test]
    fn macro_defines_typed_args_impl() {
        let args = parse(&["--path", "src", "--recursive"]);

        let typed = MacroArgs::from_args(&args).expect("typed args");

        assert_eq!(
            typed,
            MacroArgs {
                path: "src".to_string(),
                recursive: true,
            }
        );
    }
}
