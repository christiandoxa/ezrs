//! Simple typed config loading from ezrs.toml plus .env support.

use std::path::Path;

use serde::de::DeserializeOwned;

use crate::Result;

/// Loads .env into the process environment when present.
pub fn load_env() {
    let _ = dotenvy::dotenv();
}

/// Loads typed config from ezrs.toml in the current directory if it exists.
pub fn load_optional<T>() -> Result<Option<T>>
where
    T: DeserializeOwned,
{
    load_optional_from_path("ezrs.toml")
}

/// Loads typed config from a specific TOML path if it exists.
pub fn load_optional_from_path<T>(path: impl AsRef<Path>) -> Result<Option<T>>
where
    T: DeserializeOwned,
{
    let path = path.as_ref();
    if !path.exists() {
        return Ok(None);
    }

    let text = std::fs::read_to_string(path)?;
    let value = toml::from_str(&text)?;
    Ok(Some(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, serde::Deserialize, PartialEq)]
    struct Config {
        name: String,
        workers: usize,
    }

    #[test]
    fn loads_optional_toml_config() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("ezrs.toml");
        std::fs::write(&path, "name = 'demo'\nworkers = 2\n").expect("write config");

        let cfg: Config = load_optional_from_path(path)
            .expect("load")
            .expect("config present");

        assert_eq!(
            cfg,
            Config {
                name: String::from("demo"),
                workers: 2
            }
        );
    }
}
