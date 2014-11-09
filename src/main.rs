extern crate regex;

mod path {
	pub mod pathloader;
	pub mod pathfilter;
}

#[cfg(not(test))]
fn main() {
    println!("Hello, world!")
}
