#![cfg_attr(feature="clippy", feature(plugin))]

extern crate crypto;
extern crate docopt;
#[macro_use] extern crate log;
extern crate regex;
extern crate walker;

// FIXME(#19470): Remove once rust bug is fixed.
extern crate rustc_serialize;

#[cfg(test)] #[macro_use] extern crate hamcrest;
#[cfg(test)] pub mod tests;  // integration tests

pub mod duplication;
pub mod entry;
pub mod logger;
pub mod path;
pub mod utils;

use std::io::Error as IOError;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use crypto::md5::Md5;
use docopt::Docopt;
use walker::Walker;

use logger::Logger;
use path::Paths;
use entry::{Entry, EntryFactory};
use utils::FileReadOpener;

static VERSION: &'static str = "v0.1.0-dev";

static USAGE: &'static str = "
Usage: picman [--workers NUM] <source> <dest>
       picman [--workers NUM] <source>... <dest>
       picman [--workers NUM] (--help | --version)

Options:
    --workers <NUM>  uses NUM hashing worker threads [default: 4]
    -h, --help       display this help text and exit
";

#[derive(RustcDecodable)]
struct Args {
    arg_source: Vec<String>,
    arg_dest: String,
    flag_workers: usize,
}

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
    info!("using {} workers", args.flag_workers);

    let (send, recv) = channel::<Result<Entry, IOError>>();

    spawn_factory_workers(args.flag_workers, paths.all(), send);
    consume_entries(paths.all(), recv);

    info!("complete");
}

/// Populate `paths` given a root directory `path`
fn load_dir(path: PathBuf, paths: &mut Paths) {
    let dir_err_msg = format!("cannot read directory `{}`",
                              path.as_path().display());
    let path_iter = match Walker::new(path.as_path()) {
        Ok(t) => t,
        Err(e) => {
            error!("{}: {}", &dir_err_msg, e);
            return;
        }
    };
    for dir_entry in path_iter {
        match dir_entry {
            Ok(entry) => {
                if !entry.path().is_dir() {
                    paths.add(entry.path())
                }
            },
            Err(why) => warn!("cannot add path: {}", why),
        }
    }
}

/// Spawn n worker threads, each running an `EntryFactory` to create and
/// send `Entry` instances into the `sender` channel.
fn spawn_factory_workers(threads: usize, paths: &Vec<PathBuf>,
                         sender: Sender<Result<Entry, IOError>>) {
    let size: usize = paths.len() / threads;
    for paths_slice in paths.chunks(size) {
        let paths = paths_slice.to_vec();
        let local_sender = sender.clone();
        thread::spawn(|| factory_worker(paths, local_sender));
    }
}

fn factory_worker(paths: Vec<PathBuf>, sender: Sender<Result<Entry, IOError>>) {
    let mut factory = EntryFactory::new(Md5::new(), FileReadOpener::new());
    match factory.send_many(paths, &sender) {
        Ok(_) => {},
        Err(err) => {
            for maybe_path in err.failed.into_iter() {
                match maybe_path {
                    Ok(path) => error!("cannot evaluate {}", path.display()),
                    Err(why) => error!("{}", why),
                }
            }
        },
    }
    drop(sender);
}

/// Receive and consume `Entry` instances from the `recv` channel.
fn consume_entries(paths: &Vec<PathBuf>, recv: Receiver<Result<Entry, IOError>>) {
    let mut entries: Vec<Entry> = Vec::new();
    loop {
        match recv.recv() {
            Ok(maybe_entry) => match maybe_entry {
                Ok(entry) => {
                    info!("{}", entry);
                    entries.push(entry);
                },
                Err(err) => error!("{}", err),
            },
            _ => {
                if entries.len() >= paths.len() {
                    break;
                }
            },
        }
    }
}

fn set_global_logger() {
    log::set_logger(|max_level| {
        max_level.set(log::LogLevelFilter::Debug);
        Box::new(Logger::new(max_level))
    }).expect("could not create logger")
}
