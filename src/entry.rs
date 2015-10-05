use std::io::Read;
use std::path::PathBuf;
use std::vec::Vec;

use crypto::digest::Digest;


/// Represents a single file, with its computed hash digest values
pub struct Entry {
    pub path: PathBuf,
    pub hashes: Vec<String>,
}

impl Entry {
    /// Creates a new `Entry` with the given path.
    pub fn new(path: PathBuf) -> Entry {
        Entry {path: path, hashes: Vec::new()}
    }
}


pub trait ReadOpener {
    type Readable: Read;
    /// Creates a new `Readable` type, given a `PathBuf`.
    fn get_reader(PathBuf) -> Self::Readable;
}


/// Creates `Entry` instances from a `PathBuf`, and populates them
/// with an initial `Digest` hash.
pub struct EntryFactory<D, R> where D: Digest, R: ReadOpener {
    initial_digest: D,
    read_opener: R,
}

impl<D, R> EntryFactory<D, R> where D: Digest, R: ReadOpener {
    /// Creates an `EntryFactory` using the initial `Digest`, and `ReadOpener`.
    pub fn new(digest: D, read_opener: R) -> EntryFactory<D, R>
        where D: Digest, R: ReadOpener {
        EntryFactory {
            initial_digest: digest,
            read_opener: read_opener,
        }
    }

    /// Creates an `Entry`, also computing its first digest.
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

    struct CursorFactory;

    impl ReadOpener for CursorFactory {
        type Readable = Cursor<Vec<u8>>;

        fn get_reader(path: PathBuf) -> Cursor<Vec<u8>> {
            let mut cursor = Cursor::new(Vec::new());
            let data: [u8; 4] = [1, 2, 3, 4];
            assert_that(cursor.write(&data).unwrap(), is(equal_to(4)));
            cursor.set_position(0);
            cursor
        }
    }

    #[test]
    fn test_entry_factory_creates_entry() {
        let factory = EntryFactory::new(Md5::new(), CursorFactory);
    }
}
