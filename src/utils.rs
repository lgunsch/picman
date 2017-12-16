use std::io::{BufReader, BufRead, Read};
use std::io::Error as IOError;
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
pub struct HashDigester<D, R> where D: Digest, R: ReadOpener {
    digest: D,
    read_opener: R,
}

impl<D, R> HashDigester<D, R> where D: Digest, R: ReadOpener {
    pub fn new(digest: D, read_opener: R) -> HashDigester<D, R> {
        HashDigester {digest: digest, read_opener: read_opener}
    }

    /// Computes and returns a String message digest of the file.
    pub fn get_digest(&mut self, path: &PathBuf) -> Result<String, IOError> {
        let mut reader = BufReader::new(
            try!(self.read_opener.get_reader(&path))
        );

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

#[cfg(test)]
mod test {
    use super::*;
    use hamcrest::prelude::*;
    use std::io::{Cursor, Read, Write};
    use std::io::Error as IOError;
    use std::path::PathBuf;
    use std::vec::Vec;

    use crypto::md5::Md5;
    use crypto::digest::Digest;

    struct CursorReadOpener;

    impl ReadOpener for CursorReadOpener {
        type Readable = Cursor<Vec<u8>>;

        fn get_reader(&mut self, path: &PathBuf) -> Result<Cursor<Vec<u8>>, IOError> {
            let mut cursor = Cursor::new(Vec::new());
            // use path &str as file contents for testing
            let data: &[u8] = path.as_path().to_str().unwrap().as_bytes();
            assert_that!(cursor.write(&data).unwrap(),
                         is(equal_to(data.len())));
            cursor.set_position(0);
            Ok(cursor)
        }
    }

    #[test]
    fn test_hash_digester_returns_hash() {
        let path = PathBuf::from("my/test/file-name");
        let expected = create_expected_hash(&path);

        let mut digester = HashDigester::new(Md5::new(), CursorReadOpener);
        assert_that!(digester.get_digest(&path).unwrap(), is(equal_to(expected)));
    }

    #[test]
    fn test_hash_digester_resets_hash() {
        let path = PathBuf::from("my/test/file-name");
        let expected = create_expected_hash(&path);

        let mut digester = HashDigester::new(Md5::new(), CursorReadOpener);
        digester.get_digest(&path).unwrap();

        // This second call will fail if initial_digest is not reset.
        assert_that!(digester.get_digest(&path).unwrap(), is(equal_to(expected)));

    }

    fn create_expected_hash(path: &PathBuf) -> String {
        let mut buf = Vec::new();
        let mut reader = CursorReadOpener.get_reader(&path).unwrap();
        assert!(reader.read_to_end(&mut buf).is_ok());

        let mut md5 = Md5::new();
        md5.input(&mut buf[..]);

        return md5.result_str();
    }
}
