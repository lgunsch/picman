use std::vec::Vec;
use std::boxed::Box;
use std::string::String;

struct PathLoader {
    paths: Vec<Box<String>>
}

impl PathLoader {
    fn new() -> PathLoader {
        PathLoader { paths: Vec::new() }
    }

    fn add(&mut self, path: Box<String>) {
        self.paths.push(path);
    }

    fn all(&self) -> &Vec<Box<String>> {
        &self.paths
    }
}

#[test]
fn test_path_loader_add_path() {
    let mut loader = PathLoader::new();
    loader.add(box "/images/".to_string());
    loader.add(box "/path/".to_string());
    assert!(*loader.all() == vec![box "/images/".to_string(),
                                  box "/path/".to_string()]);
}
