#![feature(fs_walk)]
#![feature(plugin)]

extern crate regex;

#[cfg(test)] extern crate hamcrest;

pub mod path;

use std::error::Error;
use std::fs::walk_dir;
use std::path::PathBuf;
use path::Paths;

#[allow(dead_code)]  // TODO: Remove once a patch for #12327 lands
fn main() {
    let mut paths = Paths::new();

    // Just 2 paths for testing...
    load_dir(PathBuf::from("/var/tmp"), &mut paths);
    load_dir(PathBuf::from("/root/"), &mut paths);

   println!("Scanned in {} paths...", paths.count());
}

fn load_dir(path: PathBuf, paths: &mut Paths) {
    let dir_err_msg = format!("cannot read directory `{}`",
                              path.as_path().to_string_lossy());
    let path_iter = match walk_dir(path) {
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