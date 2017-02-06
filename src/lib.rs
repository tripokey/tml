#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

pub mod errors {
    error_chain!{}
}

use errors::*;
use std::path::Path;
use std::borrow::Cow::{self, Owned, Borrowed};
use std::os::unix::ffi::OsStrExt;

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
    } else if dst.as_os_str().as_bytes().ends_with(b"/") {
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
    use std::path::{Path};

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
        let dst = "dest/";
        expand_destination(src, dst).unwrap();
    }

    #[test]
    fn expand_destination_empty_dst() {
        let src = "/test/file";
        let dst = "";
        let res = "file";
        assert_eq!(Path::new(res), expand_destination(src, dst).unwrap());
    }

    #[test]
    fn expand_destination_with_dst() {
        let src = "/test/file";
        let dst = "destination";
        let res = "destination";
        assert_eq!(Path::new(res), expand_destination(src, dst).unwrap());
    }

    #[test]
    fn expand_destination_with_dst_ending_with_sep() {
        let src = "/test/file";
        let dst = "destination/";
        let res = "destination/file";
        assert_eq!(Path::new(res), expand_destination(src, dst).unwrap());
    }
}
