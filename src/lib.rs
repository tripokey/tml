#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

pub mod errors {
    error_chain!{}
}

use errors::*;
use std::path::{Path, MAIN_SEPARATOR};
use std::borrow::Cow::{self, Owned, Borrowed};
use std::os::unix::ffi::OsStrExt;
use std::ops::Deref;

pub struct PathExt<T: AsRef<Path>>(T);

/// PartialEq for PathExt means that both files exists and have the same inode number on the same
/// device.
impl<T: AsRef<Path>, S: AsRef<Path>> PartialEq<S> for PathExt<T> {
    fn eq(&self, other: &S) -> bool {
        use std::os::unix::fs::MetadataExt;

        self.symlink_metadata()
            .and_then(|lhs| {
                other.as_ref()
                    .symlink_metadata()
                    .and_then(|rhs| Ok(lhs.ino() == rhs.ino() && lhs.dev() == rhs.dev()))
            })
            .unwrap_or(false)
    }
}

impl<T: AsRef<Path>> PathExt<T> {
    /// Remove symlink file or empty directory
    pub fn remove(&self) -> Result<()> {
        if let Ok(meta) = self.symlink_metadata() {
            let filetype = meta.file_type();

            if filetype.is_file() || filetype.is_symlink() {
                std::fs::remove_file(self)
                    .chain_err(|| format!("Failed to remove '{}'", self.display()))?;
            } else if filetype.is_dir() {
                std::fs::remove_dir(self)
                    .chain_err(|| format!("Failed to remove '{}'", self.display()))?;
            } else {
                bail!("unknown filetype for '{}'", self.display())
            }
        }

        Ok(())
    }
}

impl<T: AsRef<Path>> From<T> for PathExt<T> {
    fn from(other: T) -> Self {
        PathExt(other)
    }
}

impl<T: AsRef<Path>> Deref for PathExt<T> {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T: AsRef<Path>> AsRef<Path> for PathExt<T> {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}


/// Expand dst based on src.
///
/// dst will be expanded according to the following rules.
///
/// # Rules
///
/// * The file_name of src will be returned if dst is empty.
/// * The file_name of src will be appended to dst and then returned if dst ends with /.
/// * Otherwise dst will be returned.
pub fn expand_destination<'a, S, D>(src: &'a S, dst: &'a D) -> Result<Cow<'a, Path>>
    where S: AsRef<Path> + ?Sized,
          D: AsRef<Path> + ?Sized
{
    let src = src.as_ref();
    let dst = dst.as_ref();
    if dst.as_os_str().is_empty() {
        src.file_name()
            .and_then(|basename| Some(Borrowed(Path::new(basename))))
            .ok_or(format!("Failed to find basename of {}", src.display()).into())
    } else if dst.as_os_str().as_bytes().ends_with(&[MAIN_SEPARATOR as u8]) {
        src.file_name()
            .and_then(|basename| Some(Owned(Path::new(dst).join(basename))))
            .ok_or(format!("Failed to find basename of {}", src.display()).into())
    } else {
        Ok(Borrowed(dst))
    }
}

#[cfg(test)]
mod tests {
    use super::expand_destination;
    use std::path::{Path, MAIN_SEPARATOR};

    trait SlashToSep {
        fn slash_to_sep(&self) -> String;
    }

    impl SlashToSep for str {
        fn slash_to_sep(&self) -> String {
            self.replace("/", &MAIN_SEPARATOR.to_string())
        }
    }

    #[test]
    #[should_panic]
    fn expand_destination_empty_src_dst() {
        let src = "";
        let dst = "";
        expand_destination(src, dst).unwrap();
    }

    #[test]
    #[should_panic]
    fn expand_destination_empty_src_dst_ends_with_sep() {
        let src = "";
        let dst = "dest/".slash_to_sep();
        expand_destination(src, &dst).unwrap();
    }

    #[test]
    fn expand_destination_empty_dst() {
        let src = "/test/file".slash_to_sep();
        let dst = "";
        let res = "file";
        assert_eq!(Path::new(res), expand_destination(&src, dst).unwrap());
    }

    #[test]
    fn expand_destination_with_dst() {
        let src = "/test/file".slash_to_sep();
        let dst = "destination";
        let res = "destination";
        assert_eq!(Path::new(res), expand_destination(&src, dst).unwrap());
    }

    #[test]
    fn expand_destination_with_dst_ending_with_sep() {
        let src = "/test/file".slash_to_sep();
        let dst = "destination/".slash_to_sep();
        let res = "destination/file".slash_to_sep();
        assert_eq!(Path::new(&res), expand_destination(&src, &dst).unwrap());
    }
}
