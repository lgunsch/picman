extern crate docopt;
extern crate regex;
extern crate walker;

// FIXME(#19470): Remove once rust bug is fixed.
extern crate rustc_serialize;

#[cfg(test)] extern crate hamcrest;

pub mod path;

use std::error::Error;
use std::path::PathBuf;

use docopt::Docopt;
use walker::Walker;

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

    let args: Args = Docopt::new(USAGE)
                             .unwrap_or_else(|e| e.exit())
                             .help(true)
                             .version(Some(VERSION.to_string()))
                             .decode()
                             .unwrap_or_else(|e| e.exit());

    for path in args.arg_source.into_iter() {
        load_dir(PathBuf::from(path), &mut paths);
    }

    println!("Scanned in {} paths...", paths.count());
}

fn load_dir(path: PathBuf, paths: &mut Paths) {
    let dir_err_msg = format!("cannot read directory `{}`",
                              path.as_path().to_string_lossy());
    let path_iter = match Walker::new(path.as_path()) {
        Ok(t) => t,
        Err(e) => {
            perror(&dir_err_msg, e);
            return;
        }
    };
    for dir_entry in path_iter {
        match dir_entry {
            Ok(entry) => paths.add(entry.path()),
            Err(why) => pwarning("cannot add path", why),
        }
    }
}

fn perror<T: Error>(msg: &str, err: T) {
    println!("ERROR: {}: {}", msg, err);
}

fn pwarning<T: Error>(msg: &str, err: T) {
    println!("Warning: {}: {}", msg, err);
}