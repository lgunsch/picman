extern crate regex;

use self::regex::Regex;
use std::collections::HashMap;
use std::boxed::Box;
use std::string::String;

struct ImagePathFilter {
    patterns: HashMap<Box<String>, Box<Regex>>
}

impl ImagePathFilter {
    fn new() -> ImagePathFilter {
        ImagePathFilter { patterns: HashMap::new() }
    }

    fn add_image_regex(&mut self, extension: Box<String>, expr: &str) {
        let re = match Regex::new(expr) {
            Ok(re) => box re,
            Err(err) => panic!("{}", err),
        };
        self.patterns.insert(extension, re);
    }

    fn is_image_path(&self, path: &str) -> bool {
        for (_, re) in self.patterns.iter() {
            if re.is_match(path) {
                return true
            }
        }
        false
    }
}

#[test]
fn test_is_path_an_image() {
    let paths = ["a.jpeg", "b.png"];

    let mut filter = ImagePathFilter::new();
    filter.add_image_regex(box "jpeg".to_string(), r"(?i)\.jpeg$");
    filter.add_image_regex(box "png".to_string(), r"(?i)\.png$");

    for path in paths.iter() {
        assert!(filter.is_image_path(*path) == true);
    }
}
