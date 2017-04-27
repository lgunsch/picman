extern crate crypto;
extern crate docopt;
#[macro_use] extern crate log;
extern crate regex;
extern crate walker;

// FIXME(#19470): Remove once rust bug is fixed.
extern crate rustc_serialize;

#[cfg(test)] #[macro_use] extern crate hamcrest;
#[cfg(test)] pub mod tests;  // integration tests

pub mod path;
pub mod entry;
pub mod logger;

use std::path::PathBuf;

use docopt::Docopt;
use walker::Walker;

use logger::Logger;
use path::Paths;

static VERSION: &'static str = "v0.1.0-dev";

static USAGE: &'static str = "
Usage: picman <source> <dest>
       picman <source>... <dest>
       picman (--help | --version)

Options:
    -h, --help  display this help text and exit
";

#[derive(RustcDecodable)]
struct Args {
    arg_source: Vec<String>,
    arg_dest: String,
}

#[cfg_attr(test, allow(dead_code))]  // TODO: Remove once a patch for #12327 lands
fn main() {
    let mut paths = Paths::new();

    // Error.exit() used below prints out an appropriate usage message,
    // which is why we don't panic instead.
    let args: Args = Docopt::new(USAGE)
                             .unwrap_or_else(|e| e.exit())
                             .help(true)
                             .version(Some(VERSION.to_string()))
                             .decode()
                             .unwrap_or_else(|e| e.exit());

    set_global_logger();

    for path in args.arg_source.into_iter() {
        load_dir(PathBuf::from(path), &mut paths);
    }

    info!("scanned in {} paths", paths.count());
    info!("destination is {}", args.arg_dest);
}

/// Populate `paths` given a root directory `path`
fn load_dir(path: PathBuf, paths: &mut Paths) {
    let dir_err_msg = format!("cannot read directory `{}`",
                              path.as_path().to_string_lossy());
    let path_iter = match Walker::new(path.as_path()) {
        Ok(t) => t,
        Err(e) => {
            error!("{}: {}", &dir_err_msg, e);
            return;
        }
    };
    for dir_entry in path_iter {
        match dir_entry {
            Ok(entry) => paths.add(entry.path()),
            Err(why) => warn!("cannot add path: {}", why),
        }
    }
}

fn set_global_logger() {
    log::set_logger(|max_level| {
        max_level.set(log::LogLevelFilter::Debug);
        Box::new(Logger::new(max_level))
    }).expect("could not create logger")
}
