use std::vec::Vec;
use std::string::String;

struct PathLoader {
    paths: Vec<String>
}

impl PathLoader {
    fn new() -> PathLoader {
        PathLoader { paths: Vec::new() }
    }

    fn add(&mut self, path: String) {
        self.paths.push(path);
    }

    fn all(&self) -> &Vec<String> {
        &self.paths
    }
}

#[test]
fn test_path_loader_add_path() {
    let mut loader = PathLoader::new();
    loader.add("/images/".to_string());
    loader.add("/path/".to_string());
    assert!(*loader.all() == vec!["/images/".to_string(),
                                  "/path/".to_string()]);
}
