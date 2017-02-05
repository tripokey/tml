#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

pub mod errors {
    error_chain!{}
}

use errors::*;
use std::path::{Path, PathBuf};

/// Creates a new symbolic link on the filesystem.
///
/// The dst path will be a symbolic link pointing to the src path.
///
/// # Note
///
/// * Parent directories for dst will be created if missing.
/// * If dst is empty it will default to the basename of src.
/// * The basename of src will be appended to dst if dst ends with '/'.
pub fn symlink(src: &str, dst: Option<&str>) -> Result<()> {
    let src = Path::new(src);
    let dst = match dst {
        Some(dst) if dst.ends_with("/") => {
            src.file_name()
                .and_then(|basename| Some(Path::new(dst).join(basename)))
                .ok_or(format!("Failed to find basename of {}", src.display()))
        }
        Some(dst) => Ok(PathBuf::from(dst)),
        None => {
            src.file_name()
                .and_then(|basename| Some(PathBuf::from(basename)))
                .ok_or(format!("Failed to find basename of {}", src.display()))
        }
    }?;

    // Create missing parent directories for dst
    if let Some(dir) = dst.parent() {
        std::fs::create_dir_all(dir)
            .chain_err(|| format!("Failed to create directory {}", dir.display()))?;
    }

    // Create dst
    std::os::unix::fs::symlink(&src, &dst)
        .chain_err(|| format!("Failed to create {}", dst.display()))
}
