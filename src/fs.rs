//! Async file helpers for CLI tools and file processors.

use std::{
    fs::OpenOptions,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::Result;
use serde::{Serialize, de::DeserializeOwned};
use tokio::io::AsyncWriteExt;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// File helper handle exposed through Context.
#[derive(Clone, Copy, Debug, Default)]
pub struct Fs;

impl Fs {
    /// Reads a UTF-8 file into a string.
    pub async fn read_to_string(&self, path: impl AsRef<Path>) -> Result<String> {
        Ok(tokio::fs::read_to_string(path).await?)
    }

    /// Writes a string to a file, creating parent directories when needed.
    pub async fn write_string(&self, path: impl AsRef<Path>, data: impl AsRef<str>) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(path, data.as_ref()).await?;
        Ok(())
    }

    /// Atomically writes a string by creating a temporary sibling and renaming it into place.
    pub async fn atomic_write_string(
        &self,
        path: impl AsRef<Path>,
        data: impl AsRef<str>,
    ) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            tokio::fs::create_dir_all(parent).await?;
        }

        let (temp, mut file) = loop {
            let temp = temp_sibling(path);
            match tokio::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp)
                .await
            {
                Ok(file) => break (temp, file),
                Err(err) if err.kind() == ErrorKind::AlreadyExists => continue,
                Err(err) => return Err(err.into()),
            }
        };

        let result = async {
            file.write_all(data.as_ref().as_bytes()).await?;
            file.sync_all().await?;
            drop(file);
            tokio::fs::rename(&temp, path).await?;
            Ok(())
        }
        .await;

        if result.is_err() {
            let _ = tokio::fs::remove_file(&temp).await;
        }

        result
    }

    /// Reads JSON into a typed value.
    pub async fn read_json<T>(&self, path: impl AsRef<Path>) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let text = self.read_to_string(path).await?;
        Ok(serde_json::from_str(&text)?)
    }

    /// Writes a typed value as pretty JSON using an atomic write.
    pub async fn write_json<T>(&self, path: impl AsRef<Path>, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let text = serde_json::to_string_pretty(value)?;
        self.atomic_write_string(path, format!("{text}\n")).await
    }

    /// Reads TOML into a typed value.
    pub async fn read_toml<T>(&self, path: impl AsRef<Path>) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let text = self.read_to_string(path).await?;
        Ok(toml::from_str(&text)?)
    }

    /// Writes a typed value as TOML using an atomic write.
    pub async fn write_toml<T>(&self, path: impl AsRef<Path>, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let text = toml::to_string(value).map_err(|err| crate::Error::msg(err.to_string()))?;
        self.atomic_write_string(path, text).await
    }

    /// Tries to create a lock file. The file is removed when the returned guard is dropped.
    pub fn try_lock(&self, path: impl AsRef<Path>) -> Result<FileLock> {
        let path = path.as_ref();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new().write(true).create_new(true).open(path)?;
        Ok(FileLock {
            path: path.to_path_buf(),
            file: Some(file),
        })
    }

    /// Returns true when the path exists.
    pub fn exists(&self, path: impl AsRef<Path>) -> bool {
        path.as_ref().exists()
    }

    /// Walks a directory tree and returns discovered paths.
    pub fn walk(&self, path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        for entry in walkdir::WalkDir::new(path) {
            let entry = entry.map_err(|err| crate::Error::msg(err.to_string()))?;
            paths.push(entry.path().to_path_buf());
        }
        Ok(paths)
    }
}

/// RAII guard for a lock file created with [`Fs::try_lock`].
#[derive(Debug)]
pub struct FileLock {
    path: PathBuf,
    file: Option<std::fs::File>,
}

impl FileLock {
    /// Returns the lock file path.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        drop(self.file.take());
        let _ = std::fs::remove_file(&self.path);
    }
}

fn temp_sibling(path: &Path) -> PathBuf {
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("file");
    let temp_name = format!(".{name}.tmp.{}.{}", std::process::id(), counter);
    path.with_file_name(temp_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
    struct State {
        name: String,
        count: u32,
    }

    #[tokio::test]
    async fn fs_read_write_exists_walk() {
        let dir = tempfile::tempdir().expect("temp dir");
        let file = dir.path().join("nested/file.txt");
        let fs = Fs;

        fs.write_string(&file, "hello").await.expect("write");
        assert!(fs.exists(&file));
        assert_eq!(fs.read_to_string(&file).await.expect("read"), "hello");

        let paths = fs.walk(dir.path()).expect("walk");
        assert!(paths.iter().any(|path| path.ends_with("file.txt")));
    }

    #[tokio::test]
    async fn atomic_write_creates_parent_and_replaces_file() {
        let dir = tempfile::tempdir().expect("temp dir");
        let file = dir.path().join("nested/state.txt");
        let fs = Fs;

        fs.atomic_write_string(&file, "first")
            .await
            .expect("first write");
        fs.atomic_write_string(&file, "second")
            .await
            .expect("second write");

        assert_eq!(fs.read_to_string(&file).await.expect("read"), "second");
        let leftovers = fs.walk(dir.path()).expect("walk");
        assert!(!leftovers.iter().any(|path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().contains(".tmp."))
        }));
    }

    #[tokio::test]
    async fn json_round_trip() {
        let dir = tempfile::tempdir().expect("temp dir");
        let file = dir.path().join("state.json");
        let fs = Fs;
        let state = State {
            name: "worker".to_string(),
            count: 2,
        };

        fs.write_json(&file, &state).await.expect("write json");
        let loaded: State = fs.read_json(&file).await.expect("read json");

        assert_eq!(loaded, state);
    }

    #[tokio::test]
    async fn toml_round_trip() {
        let dir = tempfile::tempdir().expect("temp dir");
        let file = dir.path().join("state.toml");
        let fs = Fs;
        let state = State {
            name: "worker".to_string(),
            count: 3,
        };

        fs.write_toml(&file, &state).await.expect("write toml");
        let loaded: State = fs.read_toml(&file).await.expect("read toml");

        assert_eq!(loaded, state);
    }

    #[test]
    fn lock_contention_fails_while_lock_is_held() {
        let dir = tempfile::tempdir().expect("temp dir");
        let file = dir.path().join("state.lock");
        let fs = Fs;

        let _lock = fs.try_lock(&file).expect("first lock");
        let second = fs.try_lock(&file);

        assert!(second.is_err());
    }

    #[test]
    fn lock_drop_removes_lock_file() {
        let dir = tempfile::tempdir().expect("temp dir");
        let file = dir.path().join("state.lock");
        let fs = Fs;

        {
            let lock = fs.try_lock(&file).expect("lock");
            assert_eq!(lock.path(), file.as_path());
            assert!(file.exists());
        }

        assert!(!file.exists());
        fs.try_lock(&file).expect("lock after drop");
    }
}
