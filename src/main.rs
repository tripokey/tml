#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
extern crate clap;
extern crate tml;

use clap::{Arg, App, ArgMatches};
use std::path::Path;
use std::borrow::Cow::{Owned, Borrowed};
use tml::errors::*;

const SOURCE: &'static str = "SOURCE";
const DESTINATION: &'static str = "DESTINATION";
const NO_VERIFY: &'static str = "NO_VERIFY";
const NAME: &'static str = "tml";
const FORCE: &'static str = "FORCE";

struct PathExt<T: AsRef<Path>>(T);

/// PartialEq for PathExt means that both files exists and have the same inode number on the same
/// device.
impl<T: AsRef<Path>> PartialEq for PathExt<T> {
    fn eq(&self, other: &Self) -> bool {
        use std::os::unix::fs::MetadataExt;

        self.0
            .as_ref()
            .symlink_metadata()
            .and_then(|lhs| {
                other.0
                    .as_ref()
                    .symlink_metadata()
                    .and_then(|rhs| Ok(lhs.ino() == rhs.ino() && lhs.dev() == rhs.dev()))
            })
            .unwrap_or(false)
    }
}

impl<T: AsRef<Path>> PathExt<T> {
    /// Remove symlink file or empty directory
    fn remove(&self) -> Result<()> {
        if let Ok(meta) = self.0.as_ref().symlink_metadata() {
            let filetype = meta.file_type();

            if filetype.is_file() || filetype.is_symlink() {
                std::fs::remove_file(self.0.as_ref())
                    .chain_err(|| format!("Failed to remove '{}'", self.0.as_ref().display()))?;
            } else if filetype.is_dir() {
                std::fs::remove_dir(self.0.as_ref())
                    .chain_err(|| format!("Failed to remove '{}'", self.0.as_ref().display()))?;
            } else {
                bail!("unknown filetype for '{}'", self.0.as_ref().display())
            }
        }

        Ok(())
    }
}

fn main() {
    if let Err(ref e) = run() {
        use std::io::Write;
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(errmsg);
        }

        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
        }

        ::std::process::exit(1);
    }
}

fn run() -> tml::errors::Result<()> {
    handle_args(&App::new(NAME)
        .version("0.2.0")
        .author("Michael Leandersson <tripokey@gmail.com>")
        .about(format!("{} creates a symbolic link to {} at {}, creating parent directories as \
                        needed.\n{} defaults to the basename of {} if omitted.\nThe basename of \
                        {} will be appended to {} if {} ends with a '/'.",
                       NAME,
                       SOURCE,
                       DESTINATION,
                       DESTINATION,
                       SOURCE,
                       SOURCE,
                       DESTINATION,
                       DESTINATION)
            .as_ref())
        .arg(Arg::with_name(SOURCE)
            .help("the target of the link")
            .required(true)
            .index(1))
        .arg(Arg::with_name(DESTINATION)
            .help("The destination path to create")
            .index(2))
        .arg(Arg::with_name(NO_VERIFY)
            .short("n")
            .help(format!("do not verify that {} exists", SOURCE).as_ref()))
        .arg(Arg::with_name(FORCE)
            .short("f")
            .help(format!("remove existing {}", DESTINATION).as_ref()))
        .get_matches())

}

fn handle_args(matches: &ArgMatches) -> tml::errors::Result<()> {
    let src = matches.value_of(SOURCE).ok_or(format!("{} argument missing", SOURCE))?;
    let dst = tml::expand_destination(src, matches.value_of(DESTINATION).unwrap_or(""))?;
    let verify = !matches.is_present(NO_VERIFY);
    let force = matches.is_present(FORCE);

    // Create missing parent directories for dst
    if let Some(dir) = dst.parent() {
        std::fs::create_dir_all(dir)
            .chain_err(|| format!("Failed to create directory {}", dir.display()))?;
    }

    // Verify that src exists
    if verify {
        verify_src_from_dst(src, &dst)?;
    }

    // Remove dst
    if force {
        remove_dst_if_not_src(src, &dst)?;
    }

    // Create dst
    std::os::unix::fs::symlink(&src, &dst)
        .chain_err(|| format!("Failed to create {}", dst.display()))
}

fn verify_src_from_dst<S, D>(src: &S, dst: &D) -> tml::errors::Result<()>
    where S: AsRef<Path> + ?Sized,
          D: AsRef<Path> + ?Sized
{
    let src = src.as_ref();
    let dst = dst.as_ref();

    let path = if src.is_relative() {
        match dst.parent() {
            Some(cwd) => Owned(cwd.join(src)),
            None => Borrowed(src),
        }
    } else {
        Borrowed(src)
    };

    if !path.exists() {
        bail!("{} does not exist", path.display());
    }

    Ok(())
}

fn remove_dst_if_not_src<S, D>(src: &S, dst: &D) -> Result<()>
    where S: AsRef<Path> + ?Sized,
          D: AsRef<Path> + ?Sized
{
    let src = PathExt(src.as_ref());
    let dst = PathExt(dst.as_ref());

    if src != dst {
        dst.remove()
    } else {
        let src = src.0;
        let dst = dst.0;
        bail!("'{}' and '{}' are the same file",
              src.display(),
              dst.display())
    }
}
