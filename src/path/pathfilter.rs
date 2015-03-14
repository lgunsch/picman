use regex::{Regex, Error};
use std::collections::HashMap;
use std::path::Path;
use std::vec::Vec;

pub struct PathExtensionFilter {
    extension_patterns: HashMap<String, Regex>
}

impl PathExtensionFilter {
    pub fn new() -> PathExtensionFilter {
        PathExtensionFilter { extension_patterns: HashMap::new() }
    }

    pub fn add_jpeg(&mut self) -> Result<(), Error> {
        try!(self.add_extension_regex("jpeg".to_string(),
                                      r"(?i)jpe?g$".to_string()));
        return Ok(());
    }

    pub fn add_png(&mut self) -> Result<(), Error> {
        try!(self.add_extension_regex("png".to_string(),
                                      r"(?i)png$".to_string()));
        return Ok(());
    }

    pub fn add_bmp(&mut self) -> Result<(), Error> {
        try!(self.add_extension_regex("bmp".to_string(),
                                      r"(?i)bmp$".to_string()));
        return Ok(());
    }

    pub fn add_extension_regex(&mut self, extension: String, expr: String) -> Result<(), Error> {
        let re = try!(Regex::new(&expr));
        self.extension_patterns.insert(extension, re);
        return Ok(());
    }

    pub fn add_many_extension_regex(&mut self, expressions: Vec<(String, String)>) -> Result<(), Error> {
        for (name, expr) in expressions.into_iter() {
            try!(self.add_extension_regex(name, expr));
        }
        return Ok(());
    }

    pub fn is_match(&self, path: &Path) -> bool {
        for (_, re) in self.extension_patterns.iter() {
            let ext = match path.extension() {
                Some(ext) => match ext.to_str() {
                    Some(s) => s,
                    None => continue  // not a valid match, its not even unicode
                },
                None => ""
            };
            if re.is_match(ext) {
                return true
            }
        }
        false
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::Path;
    use hamcrest::{assert_that, is, equal_to};

    static PATHS: [&'static str; 7] = ["a.jpeg", "b.jpg", "c.JpEG",
                                       "d.png", "e.PnG", "f.bmp",
                                       "f.BmP"];

    #[test]
    fn test_is_match() {
        let mut filter = PathExtensionFilter::new();

        assert!(filter.add_jpeg().is_ok());
        assert!(filter.add_png().is_ok());
        assert!(filter.add_bmp().is_ok());

        for p in PATHS.iter() {
            let path = Path::new(p);
            assert_that(filter.is_match(path), is(equal_to(true)));
        }
    }

    #[test]
    fn test_not_match() {
        let mut filter = PathExtensionFilter::new();

        assert!(filter.add_extension_regex("txt".to_string(), r"^(?i)txt$".to_string()).is_ok());
        for p in PATHS.iter() {
            let path = Path::new(p);
            assert_that(filter.is_match(path), is(equal_to(false)));
        }
    }

    #[test]
    fn test_bad_regex_error() {
        let mut filter = PathExtensionFilter::new();
        assert!(filter.add_extension_regex("bmp".to_string(), r"($".to_string()).is_err());
    }
}
