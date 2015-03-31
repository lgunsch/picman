#![feature(convert)]
#![feature(plugin)]

extern crate regex;

#[cfg(test)] extern crate hamcrest;

pub mod path;

#[cfg(not(test))] use std::error::Error;
#[cfg(not(test))] use std::fs::walk_dir;
#[cfg(not(test))] use std::path::PathBuf;
#[cfg(not(test))] use path::Paths;

#[cfg(not(test))]
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
            Err(e) => pwarning("cannot add path", e),
        }
    }

   println!("Scanned in {} paths...", paths.count());
}

#[cfg(not(test))]
fn perror<T: Error>(msg: &str, err: T) {
    println!("ERROR: {}: {}", msg, err);
}

#[cfg(not(test))]
fn pwarning<T: Error>(msg: &str, err: T) {
    println!("WARNING: {}: {}", msg, err);
}