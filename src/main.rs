#![feature(plugin)]
extern crate regex;

#[cfg(test)]
extern crate hamcrest;

pub mod path;

#[cfg(not(test))]
fn main() {
    println!("Hello, world!")
}
