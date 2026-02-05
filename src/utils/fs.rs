use std::io;
use std::path::Path;

use tokio::fs;

/// Create an empty directory, removing any existing files or directories
pub async fn create_clean_dir(path: impl AsRef<Path>) -> Result<(), io::Error> {
    let path = path.as_ref();

    clean_path(path).await?;
    fs::create_dir_all(path).await?;

    Ok(())
}

/// Remove a file or directory
pub async fn clean_path(path: impl AsRef<Path>) -> Result<(), io::Error> {
    let path = path.as_ref();

    if path.exists() {
        if path.is_dir() {
            fs::remove_dir_all(path).await?;
        } else {
            fs::remove_file(path).await?;
        }
    }
    Ok(())
}
