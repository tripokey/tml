#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
extern crate clap;
extern crate tml;

use clap::{Arg, App, ArgMatches};
use std::path::Path;
use tml::errors::*;

const SOURCE: &'static str = "SOURCE";
const DESTINATION: &'static str = "DESTINATION";
const NO_VERIFY: &'static str = "NO_VERIFY";
const NAME: &'static str = "tml";

fn main() {
    if let Err(ref e) = run() {
        use ::std::io::Write;
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
        .version("0.1.0")
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
        .get_matches())

}

fn handle_args(matches: &ArgMatches) -> tml::errors::Result<()> {
    let src = matches.value_of(SOURCE).ok_or(format!("{} argument missing", SOURCE))?;
    let dst = tml::expand_destination(src, matches.value_of(DESTINATION).unwrap_or(""))?;
    let verify = !matches.is_present(NO_VERIFY);

    // Verify that src exists
    if verify && !Path::new(src).exists() {
        bail!("{} does not exist", src);
    }

    // Create missing parent directories for dst
    if let Some(dir) = dst.parent() {
        std::fs::create_dir_all(dir)
            .chain_err(|| format!("Failed to create directory {}", dir.display()))?;
    }

    // Create dst
    std::os::unix::fs::symlink(&src, &dst)
        .chain_err(|| format!("Failed to create {}", dst.display()))
}
