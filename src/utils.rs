use std::collections::HashMap;
use std::io::{BufRead, BufReader, Cursor, Read};
use std::io::Error as IOError;
use std::io::ErrorKind;
use std::fs::File;
use std::path::PathBuf;

use crypto::digest::Digest;

/// A trait providing a single `get_reader` function to return a `Readable`
pub trait ReadOpener {
    type Readable: Read;
    /// Creates a new `Readable` type, given a `PathBuf`.
    fn get_reader(&mut self, &PathBuf) -> Result<Self::Readable, IOError>;
}

/// `ReadOpener` implementation using `File::open`
pub struct FileReadOpener;

impl FileReadOpener {
    pub fn new() -> FileReadOpener {
        FileReadOpener
    }
}

impl ReadOpener for FileReadOpener {
    type Readable = File;

    fn get_reader(&mut self, path: &PathBuf) -> Result<File, IOError> {
        File::open(&path)
    }
}

/// Computes message digest of files
pub struct HashDigester<D, R>
where
    D: Digest,
    R: ReadOpener,
{
    digest: D,
    read_opener: R,
}

impl<D, R> HashDigester<D, R>
where
    D: Digest,
    R: ReadOpener,
{
    pub fn new(digest: D, read_opener: R) -> HashDigester<D, R> {
        HashDigester {
            digest: digest,
            read_opener: read_opener,
        }
    }

    /// Computes and returns a String message digest of the file.
    pub fn get_digest(&mut self, path: &PathBuf) -> Result<String, IOError> {
        let mut reader = BufReader::new(try!(self.read_opener.get_reader(&path)));

        loop {
            let nread = {
                let buf = try!(reader.fill_buf());
                self.digest.input(buf);
                buf.len()
            };
            reader.consume(nread);
            if nread == 0 {
                break;
            }
        }

        let digest = self.digest.result_str();
        self.digest.reset();
        Ok(digest)
    }
}

// `ReadOpener` implementation using a map of path to `Cursor`
pub struct CursorReadOpener {
    cursors: HashMap<PathBuf, Cursor<Vec<u8>>>,
}

impl CursorReadOpener {
    pub fn new() -> CursorReadOpener {
        CursorReadOpener {
            cursors: HashMap::new(),
        }
    }

    pub fn add_path(&mut self, path: &PathBuf, cursor: Cursor<Vec<u8>>) {
        self.cursors.insert(path.clone(), cursor);
    }
}

impl ReadOpener for CursorReadOpener {
    type Readable = Cursor<Vec<u8>>;

    fn get_reader(&mut self, path: &PathBuf) -> Result<Cursor<Vec<u8>>, IOError> {
        let cursor = match self.cursors.get_mut(path) {
            Some(cursor) => cursor,
            None => {
                let msg = format!("cursor not found {}", path.as_path().display());
                return Err(IOError::new(ErrorKind::NotConnected, msg));
            }
        };
        cursor.set_position(0);
        Ok(cursor.clone())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use hamcrest::prelude::*;
    use std::io::{Cursor, Write};
    use std::path::PathBuf;
    use std::vec::Vec;

    use crypto::md5::Md5;
    use crypto::digest::Digest;

    #[test]
    fn test_hash_digester_returns_hash() {
        let path = PathBuf::from("my/test/file-name");
        let (expected, opener) = create_expected_hash_and_opener(&path);

        let mut digester = HashDigester::new(Md5::new(), opener);
        assert_that!(digester.get_digest(&path).unwrap(), is(equal_to(expected)));
    }

    #[test]
    fn test_hash_digester_resets_hash() {
        let path = PathBuf::from("my/test/file-name");
        let (expected, opener) = create_expected_hash_and_opener(&path);

        let mut digester = HashDigester::new(Md5::new(), opener);
        digester.get_digest(&path).unwrap();

        // This second call will fail if initial_digest is not reset.
        assert_that!(digester.get_digest(&path).unwrap(), is(equal_to(expected)));
    }

    fn create_expected_hash_and_opener(path: &PathBuf) -> (String, CursorReadOpener) {
        // use path &str as file contents for testing
        let mut cursor = Cursor::new(Vec::new());
        let data: &[u8] = path.as_path().to_str().unwrap().as_bytes();
        assert_that!(cursor.write(&data).unwrap(), is(equal_to(data.len())));

        let mut md5 = Md5::new();
        md5.input(&data);

        let mut opener = CursorReadOpener::new();
        opener.add_path(&path, cursor);

        return (md5.result_str(), opener);
    }
}
