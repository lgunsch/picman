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
    let path_iter = match walk_dir(PathBuf::from("/var/tmp")) {
        Ok(t) => t,
        Err(e) => {
            perror("cannot read directory", e);
            return;
        }
    };
    for dir_entry in path_iter {
        match dir_entry {
            Ok(de) => paths.add(de.path()),
            Err(why) => pwarning("cannot add path", why),
        }
    }

   println!("Scanned in {} paths...", paths.count());
}

fn perror<T: Error>(msg: &str, err: T) {
    println!("ERROR: {}: {}", msg, err);
}

fn pwarning<T: Error>(msg: &str, err: T) {
    println!("WARNING: {}: {}", msg, err);
}