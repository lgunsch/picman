use std::io::{BufRead, BufReader, Read, Error};
use std::path::PathBuf;
use std::vec::Vec;

use crypto::digest::Digest;


/// Represents a single file, with its computed hash digest values
#[derive(Debug, Eq, PartialEq)]
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
    fn get_reader(&mut self, &PathBuf) -> Self::Readable;
}


/// Creates `Entry` instances from a `PathBuf`, and populates them
/// with an initial `Digest` hash.
pub struct EntryFactory<D, R> where D: Digest, R: ReadOpener {
    initial_digest: D,
    read_opener: R,
}

impl<D, R> EntryFactory<D, R> where D: Digest, R: ReadOpener {
    /// Creates an `EntryFactory` using the initial `Digest`, and `ReadOpener`.
    pub fn new(digest: D, read_opener: R) -> EntryFactory<D, R> {
        EntryFactory {
            initial_digest: digest,
            read_opener: read_opener,
        }
    }

    /// Creates an `Entry`, also computing its first digest.
    pub fn create(&mut self, path: PathBuf) -> Result<Entry, Error>  {
        let mut reader = BufReader::new(self.read_opener.get_reader(&path));

        loop {
            let nread = {
                let buf = try!(reader.fill_buf());
                self.initial_digest.input(buf);
                buf.len()
            };
            reader.consume(nread);
            if nread == 0 {
                break;
            }
        }

        let mut hashes = Vec::new();
        hashes.push(self.initial_digest.result_str());
        self.initial_digest.reset();

        Ok(Entry {path: path, hashes: hashes})
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

        fn get_reader(&mut self, path: &PathBuf) -> Cursor<Vec<u8>> {
            let mut cursor = Cursor::new(Vec::new());
            // use path &str as file contents for testing
            let data: &[u8] = path.as_path().to_str().unwrap().as_bytes();
            assert_that(cursor.write(&data).unwrap(),
                        is(equal_to(data.len())));
            cursor.set_position(0);
            cursor
        }
    }

    #[test]
    fn test_entry_factory_creates_entry() {
        let path = PathBuf::from("my/test/file-name");
        let expected = create_expected_entry(&path);

        let mut factory = EntryFactory::new(Md5::new(), CursorFactory);
        let entry = factory.create(path).unwrap();

        assert_that(entry, is(equal_to(expected)));
    }

    #[test]
    fn test_entry_factory_resets_digest() {
        let path = PathBuf::from("my/test/file-name");
        let expected = create_expected_entry(&path);

        let mut factory = EntryFactory::new(Md5::new(), CursorFactory);

        factory.create(path.clone()).unwrap();
        // This second create will fail if initial_digest is not reset.
        let entry = factory.create(path).unwrap();
        assert_that(entry, is(equal_to(expected)));
    }

    fn create_expected_entry(path: &PathBuf) -> Entry {
        let mut buf = Vec::new();
        let mut reader = CursorFactory.get_reader(&path);
        assert!(reader.read_to_end(&mut buf).is_ok());

        let mut md5 = Md5::new();
        md5.input(&mut buf[..]);

        Entry {path: path.clone(),
               hashes: vec![md5.result_str()]}
    }
}
