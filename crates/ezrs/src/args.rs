use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use crate::{Error, Result};

/// Dynamic command arguments exposed through Context.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Args {
    values: HashMap<String, String>,
    flags: HashSet<String>,
    positionals: Vec<String>,
}

impl Args {
    pub(crate) fn parse(tokens: &[String]) -> Self {
        let mut args = Self::default();
        let mut positional_index = 0_usize;
        let mut index = 0_usize;

        while index < tokens.len() {
            let token = &tokens[index];

            if token == "--" {
                index += 1;
                while index < tokens.len() {
                    args.insert_positional(positional_index, tokens[index].clone());
                    positional_index += 1;
                    index += 1;
                }
                break;
            }

            if let Some(raw) = token.strip_prefix("--") {
                if raw.is_empty() {
                    index += 1;
                    continue;
                }

                if let Some((key, value)) = raw.split_once('=') {
                    args.values.insert(key.to_string(), value.to_string());
                    args.flags.insert(key.to_string());
                    index += 1;
                    continue;
                }

                if index + 1 < tokens.len() && !tokens[index + 1].starts_with("--") {
                    args.values
                        .insert(raw.to_string(), tokens[index + 1].clone());
                    args.flags.insert(raw.to_string());
                    index += 2;
                    continue;
                }

                args.flags.insert(raw.to_string());
                args.values.insert(raw.to_string(), String::from("true"));
                index += 1;
                continue;
            }

            args.insert_positional(positional_index, token.clone());
            positional_index += 1;
            index += 1;
        }

        args
    }

    fn insert_positional(&mut self, index: usize, value: String) {
        let key = index.to_string();
        self.values.insert(key, value.clone());
        self.positionals.push(value);
    }

    /// Returns a named or positional argument by key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    /// Parses a named or positional argument into a typed value.
    pub fn parse_value<T>(&self, key: &str) -> Result<T>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        self.get(key)
            .ok_or_else(|| Error::not_found(format!("argument '{key}'")))?
            .parse::<T>()
            .map_err(|err| Error::invalid_input(format!("argument '{key}': {err}")))
    }

    /// Parses a named or positional argument, returning a default when missing.
    pub fn parse_or<T>(&self, key: &str, default: T) -> Result<T>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        match self.get(key) {
            Some(value) => value
                .parse::<T>()
                .map_err(|err| Error::invalid_input(format!("argument '{key}': {err}"))),
            None => Ok(default),
        }
    }

    /// Returns a named argument or default value.
    pub fn get_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get(key).unwrap_or(default)
    }

    /// Returns true if the flag was present.
    pub fn flag(&self, key: &str) -> bool {
        self.flags.contains(key) || self.values.get(key).is_some_and(|value| value == "true")
    }

    /// Returns positional arguments in encounter order.
    pub fn positionals(&self) -> &[String] {
        &self.positionals
    }

    /// Returns a positional argument by numeric index.
    pub fn positional(&self, index: usize) -> Option<&str> {
        self.positionals.get(index).map(String::as_str)
    }

    /// Parses a positional argument by numeric index.
    pub fn parse_positional<T>(&self, index: usize) -> Result<T>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        self.positional(index)
            .ok_or_else(|| Error::not_found(format!("positional argument {index}")))?
            .parse::<T>()
            .map_err(|err| Error::invalid_input(format!("positional argument {index}: {err}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Args {
        Args::parse(
            &args
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>(),
        )
    }

    #[test]
    fn parses_boolean_flags() {
        let args = parse(&["--recursive"]);
        assert!(args.flag("recursive"));
        assert_eq!(args.get("recursive"), Some("true"));
    }

    #[test]
    fn parses_key_value_args() {
        let args = parse(&["--path", "src", "--name=Ayu"]);
        assert_eq!(args.get("path"), Some("src"));
        assert_eq!(args.get("name"), Some("Ayu"));
    }

    #[test]
    fn parses_positionals() {
        let args = parse(&["input.txt", "out.txt"]);
        assert_eq!(args.get("0"), Some("input.txt"));
        assert_eq!(args.get("1"), Some("out.txt"));
        assert_eq!(args.positionals(), &["input.txt", "out.txt"]);
        assert_eq!(args.positional(0), Some("input.txt"));
    }

    #[test]
    fn parses_typed_values() {
        let args = parse(&["--limit", "5", "42"]);

        assert_eq!(args.parse_value::<usize>("limit").expect("limit"), 5);
        assert_eq!(args.parse_or::<usize>("missing", 7).expect("default"), 7);
        assert_eq!(args.parse_positional::<u64>(0).expect("positional"), 42);
    }
}
