use regex::{Regex, Error};
use std::collections::HashMap;
use std::vec::Vec;

struct PathFilter<'a> {
    patterns: HashMap<&'a str, Regex>
}

impl<'a> PathFilter<'a> {
    fn new() -> PathFilter<'a> {
        PathFilter { patterns: HashMap::new() }
    }

    fn add_filter_regex(&mut self, name: &'a str, expr: &'a str) -> Result<(), Error> {
        let re = try!(Regex::new(expr));
        self.patterns.insert(name, re);
        return Ok(());
    }

    fn add_many_filter_regex(&mut self, expressions: Vec<(&'a str, &'a str)>) -> Result<(), Error> {
        for (name, expr) in expressions.into_iter() {
            try!(self.add_filter_regex(name, expr));
        }
        return Ok(());
    }

    fn is_match(&self, path: &str) -> bool {
        for (_, re) in self.patterns.iter() {
            if re.is_match(path) {
                return true
            }
        }
        false
    }
}

#[test]
fn test_is_match() {
    let paths = ["a.jpeg", "b.png"];

    let mut filter = PathFilter::new();
    let filter_regexs = vec![("jpeg", r"(?i)\.jpeg$"),
                             ("png", r"(?i)\.png$")];
    assert!(filter.add_many_filter_regex(filter_regexs).is_ok());

    for path in paths.iter() {
        assert_eq!(filter.is_match(*path), true);
    }
}

#[test]
fn test_not_match() {
    let paths = ["a.jpeg", "b.png"];
    let mut filter = PathFilter::new();
    assert!(filter.add_filter_regex("bmp", r"(?i)\.bmp$").is_ok());
    for path in paths.iter() {
        assert_eq!(filter.is_match(*path), false);
    }
}

#[test]
fn test_bad_regex_error() {
    let mut filter = PathFilter::new();
    // causes error: un-closed (
    assert!(filter.add_filter_regex("bmp", r"($").is_err());
}