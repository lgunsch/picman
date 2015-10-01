use std::io::BufRead;
use std::path::PathBuf;
use std::vec::Vec;

use crypto::digest::Digest;

/// Represents a single file, with its computed hash digest values
pub struct Entry {
    pub path: PathBuf,
    pub hashes: Vec<String>,
}

impl Entry {
    /// Construct a new Entry with the given path.
    pub fn new(path: PathBuf) -> Entry {
        Entry {path: path, hashes: Vec::new()}
    }
}

pub type BufReadFactory = Fn(PathBuf) -> Box<BufRead>;

/// Creates Entry instances from a path, and populates them
/// with an initial MD5 hash.
pub struct EntryFactory<'a> {
    initial_digest: Box<Digest + 'a>,
    bufread_factory: Box<BufReadFactory>,
}

impl<'a> EntryFactory<'a> {

    pub fn new(initial: Box<Digest + 'a>,
                  bufread_factory: Box<BufReadFactory>) -> EntryFactory<'a> {
        EntryFactory {
            initial_digest: initial,
            bufread_factory: bufread_factory,
        }
    }

    /// Construct a new instance of Entry, also computing its first digest.
    pub fn create(self, path: PathBuf) -> Entry {
        Entry {path: PathBuf::new(), hashes: Vec::new()}
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use hamcrest::{assert_that, is, equal_to};
    use std::io::{Cursor, Read, Write, BufRead, BufReader};
    use std::path::PathBuf;
    use std::vec::Vec;

    use crypto::md5::Md5;
    use crypto::digest::Digest;

    #[test]
    fn test_entry_factory_creates_entry() {
        let bufread_factory = |path: PathBuf| {
            let mut cursor = Cursor::new(Vec::new());
            let data: [u8; 4] = [1, 2, 3, 4];
            assert_that(cursor.write(&data).unwrap(), is(equal_to(4)));
            cursor.set_position(0);
            Box::new(BufReader::new(cursor)) as Box<BufRead + 'static>
        };
        let factory = EntryFactory::new(Box::new(Md5::new()), Box::new(bufread_factory));
    }
}
