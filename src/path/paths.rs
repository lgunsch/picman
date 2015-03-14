use path::PathExtensionFilter;
use std::path::PathBuf;
use std::vec::Vec;

pub struct Paths {
    paths: Vec<PathBuf>
}

impl Paths {
    pub fn new() -> Paths {
        Paths { paths: Vec::new() }
    }

    pub fn add(&mut self, path: PathBuf) {
        self.paths.push(path);
    }

    pub fn add_many(&mut self, paths: Vec<PathBuf>) {
        for p in paths.into_iter() {
            self.add(p);
        }
    }

    pub fn all(&self) -> &Vec<PathBuf> {
        &self.paths
    }

    pub fn apply_filter(&mut self, filter: &PathExtensionFilter) {
        self.paths.retain(|p| filter.is_match(&p))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;
    use path::PathExtensionFilter;
    use hamcrest::{assert_that, is, equal_to};

    #[test]
    fn test_path_loader_add_path() {
        let mut loader = Paths::new();
        loader.add_many(vec![PathBuf::new("/images/img.png"),
                             PathBuf::new("/path/text.txt")]);

        let expected = vec![PathBuf::new("/images/img.png"),
                            PathBuf::new("/path/text.txt")];
        assert_that(loader.all(), is(equal_to(&expected)));
    }

    #[test]
    fn test_apply_filter() {
        let mut loader = Paths::new();
        let mut filter = PathExtensionFilter::new();
        assert!(filter.add_jpeg().is_ok());
        loader.add_many(vec![PathBuf::new("a.txt"),
                             PathBuf::new("b.png"),
                             PathBuf::new("c.jpeg"),
                             PathBuf::new("d.JPEG")]);
        loader.apply_filter(&filter);

        let expected = vec![PathBuf::new("c.jpeg"), PathBuf::new("d.JPEG")];
        assert_that(loader.all(), is(equal_to(&expected)));
    }
}
