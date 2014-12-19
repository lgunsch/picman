use regex::{Regex, Error};
use std::collections::HashMap;
use std::string::String;
use std::vec::Vec;

pub struct PathFilter {
    patterns: HashMap<String, Regex>
}

impl PathFilter {
    pub fn new() -> PathFilter {
        PathFilter { patterns: HashMap::new() }
    }

    pub fn add_filter_regex(&mut self, extension: String, expr: String) -> Result<(), Error> {
        let re = try!(Regex::new(expr.as_slice()));
        self.patterns.insert(extension, re);
        return Ok(());
    }

    pub fn add_many_filter_regex(&mut self, expressions: Vec<(String, String)>) -> Result<(), Error> {
        for (name, expr) in expressions.into_iter() {
            try!(self.add_filter_regex(name, expr));
        }
        return Ok(());
    }

    pub fn is_match(&self, path: &str) -> bool {
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
    let filter_regexs = vec![("jpeg".to_string(), r"(?i)\.jpeg$".to_string()),
                             ("png".to_string(), r"(?i)\.png$".to_string())];
    assert!(filter.add_many_filter_regex(filter_regexs).is_ok());

    for path in paths.iter() {
        assert_eq!(filter.is_match(*path), true);
    }
}

#[test]
fn test_not_match() {
    let paths = ["a.jpeg", "b.png"];
    let mut filter = PathFilter::new();
    assert!(filter.add_filter_regex("bmp".to_string(), r"(?i)\.bmp$".to_string()).is_ok());
    for path in paths.iter() {
        assert_eq!(filter.is_match(*path), false);
    }
}

#[test]
fn test_bad_regex_error() {
    let mut filter = PathFilter::new();
    assert!(filter.add_filter_regex("bmp".to_string(), r"($".to_string()).is_err());
}