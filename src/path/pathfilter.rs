use regex::{Regex, Error};
use std::collections::HashMap;
use std::string::String;

struct PathFilter {
    patterns: HashMap<String, Regex>
}

impl PathFilter {
    fn new() -> PathFilter {
        PathFilter { patterns: HashMap::new() }
    }

    fn add_filter_regex(&mut self, extension: String, expr: &str) -> Result<(), Error> {
        let re = try!(Regex::new(expr));
        self.patterns.insert(extension, re);
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
    assert!(filter.add_filter_regex("jpeg".to_string(), r"(?i)\.jpeg$").is_ok());
    assert!(filter.add_filter_regex("png".to_string(), r"(?i)\.png$").is_ok());

    for path in paths.iter() {
        assert_eq!(filter.is_match(*path), true);
    }
}

#[test]
fn test_not_match() {
    let paths = ["a.jpeg", "b.png"];
    let mut filter = PathFilter::new();
    assert!(filter.add_filter_regex("bmp".to_string(), r"(?i)\.bmp$").is_ok());
    for path in paths.iter() {
        assert_eq!(filter.is_match(*path), false);
    }
}

#[test]
fn test_bad_regex_error() {
    let mut filter = PathFilter::new();
    assert!(filter.add_filter_regex("bmp".to_string(), r"($").is_err());
}