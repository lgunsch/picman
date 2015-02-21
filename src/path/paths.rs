use path::PathFilter;
use std::vec::Vec;
use std::string::String;

pub struct Paths {
    paths: Vec<String>
}

impl Paths {
    pub fn new() -> Paths {
        Paths { paths: Vec::new() }
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
        self.paths.retain(|ref p| filter.is_match(&p))
    }
}

#[cfg(test)]
mod test {
	use super::*;
	use path::PathFilter;
	use hamcrest::{assert_that, is, equal_to};

	#[test]
	fn test_path_loader_add_path() {
	    let mut loader = Paths::new();
	    loader.add_many(vec!["/images/img.png".to_string(),
	                         "/path/text.txt".to_string()]);

	    let expected = vec!["/images/img.png".to_string(),
	                        "/path/text.txt".to_string()];
	    assert_that(loader.all(), is(equal_to(&expected)));
	}

	#[test]
	fn test_apply_filter() {
	    let mut loader = Paths::new();
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
}
