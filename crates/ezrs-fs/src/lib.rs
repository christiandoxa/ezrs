//! Async file helpers for CLI tools and file processors.

use std::path::{Path, PathBuf};

use ezrs_error::Result;

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

    /// Returns true when the path exists.
    pub fn exists(&self, path: impl AsRef<Path>) -> bool {
        path.as_ref().exists()
    }

    /// Walks a directory tree and returns discovered paths.
    pub fn walk(&self, path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        for entry in walkdir::WalkDir::new(path) {
            let entry = entry.map_err(|err| ezrs_error::Error::msg(err.to_string()))?;
            paths.push(entry.path().to_path_buf());
        }
        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
