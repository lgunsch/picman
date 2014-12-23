use hamcrest::{assert_that, is, equal_to};
use path::PathFilter;
use std::vec::Vec;
use std::string::String;

pub struct PathLoader {
    paths: Vec<String>
}

impl PathLoader {
    pub fn new() -> PathLoader {
        PathLoader { paths: Vec::new() }
    }

    pub fn add(&mut self, path: String) {
        self.paths.push(path);
    }

    pub fn add_many(&mut self, paths: Vec<String>) {
        for path in paths.into_iter() {
            self.add(path);
        }
    }

    pub fn all(&self) -> &Vec<String> {
        &self.paths
    }

    pub fn apply_filter(&mut self, filter: &PathFilter) {
        self.paths.retain(|ref p| filter.is_match(p.as_slice()))
    }
}

#[test]
fn test_path_loader_add_path() {
    let mut loader = PathLoader::new();
    loader.add_many(vec!["/images/img.png".to_string(),
                         "/path/text.txt".to_string()]);

    let expected = vec!["/images/img.png".to_string(),
                        "/path/text.txt".to_string()];
    assert_that(loader.all(), is(equal_to(&expected)));
}

#[test]
fn test_apply_filter() {
    let mut loader = PathLoader::new();
    let mut filter = PathFilter::new();

    assert!(filter.add_filter_regex("jpeg".to_string(), r"(?i)\.jpeg$".to_string()).is_ok());
    loader.add_many(vec!["a.txt".to_string(),
                         "b.png".to_string(),
                         "c.jpeg".to_string(),
                         "d.JPEG".to_string()]);
    loader.apply_filter(&filter);

    let expected = vec!["c.jpeg".to_string(), "d.JPEG".to_string()];
    assert_that(loader.all(), is(equal_to(&expected)));
}
