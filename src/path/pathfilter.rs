use regex::Regex;
use std::collections::HashMap;
use std::boxed::Box;
use std::string::String;

struct PathFilter {
    patterns: HashMap<Box<String>, Box<Regex>>
}

impl PathFilter {
    fn new() -> PathFilter {
        PathFilter { patterns: HashMap::new() }
    }

    fn add_filter_regex(&mut self, extension: Box<String>, expr: &str) {
        let re = match Regex::new(expr) {
            Ok(re) => box re,
            Err(err) => panic!("{}", err),
        };
        self.patterns.insert(extension, re);
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
    filter.add_filter_regex(box "jpeg".to_string(), r"(?i)\.jpeg$");
    filter.add_filter_regex(box "png".to_string(), r"(?i)\.png$");

    for path in paths.iter() {
        assert!(filter.is_match(*path) == true);
    }
}
