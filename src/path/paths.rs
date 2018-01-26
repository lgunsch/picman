use path::PathExtensionFilter;
use std::path::PathBuf;
use std::vec::Vec;

#[derive(Default)]
pub struct Paths {
    paths: Vec<PathBuf>,
}

impl Paths {
    pub fn new() -> Paths {
        Paths { paths: Vec::new() }
    }

    pub fn add(&mut self, path: PathBuf) {
        self.paths.push(path);
    }

    pub fn add_many(&mut self, paths: Vec<PathBuf>) {
        for p in paths {
            self.add(p);
        }
    }

    pub fn all(&self) -> &Vec<PathBuf> {
        &self.paths
    }

    pub fn apply_filter(&mut self, filter: &PathExtensionFilter) {
        self.paths.retain(|p| filter.is_match(p))
    }

    pub fn count(&self) -> usize {
        self.paths.len()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;
    use path::PathExtensionFilter;
    use hamcrest::prelude::*;

    fn create() -> Paths {
        let mut loader = Paths::new();
        loader.add_many(vec![
            PathBuf::from("a.txt"),
            PathBuf::from("b.png"),
            PathBuf::from("c.jpeg"),
            PathBuf::from("d.JPEG"),
        ]);
        return loader;
    }

    #[test]
    fn test_path_loader_add_many() {
        let loader = create();
        let expected = vec![
            PathBuf::from("a.txt"),
            PathBuf::from("b.png"),
            PathBuf::from("c.jpeg"),
            PathBuf::from("d.JPEG"),
        ];
        assert_that!(loader.all(), is(equal_to(&expected)));
    }

    #[test]
    fn test_apply_filter() {
        let mut loader = create();
        let mut filter = PathExtensionFilter::new();
        assert!(filter.add_jpeg().is_ok());
        loader.apply_filter(&filter);

        let expected = vec![PathBuf::from("c.jpeg"), PathBuf::from("d.JPEG")];
        assert_that!(loader.all(), is(equal_to(&expected)));
    }

    #[test]
    fn test_count() {
        let loader = create();
        assert_that!(loader.count(), is(equal_to(4)));
    }
}
