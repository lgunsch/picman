use std::error::Error;
use std::io::{BufRead, BufReader, Read};
use std::io::Error as IOError;
use std::fmt;
use std::fs::File;
use std::path::PathBuf;
use std::sync::mpsc::{Sender, SendError};
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
    fn get_reader(&mut self, &PathBuf) -> Result<Self::Readable, IOError>;
}

#[derive(Debug)]
pub struct EntrySendError {
	pub failed: Vec<Result<PathBuf, IOError>>,
}

impl fmt::Display for EntrySendError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Hello from EntrySendError!")
	}
}

impl Error for EntrySendError {
	fn description(&self) -> &str {
		"string?"
	}
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
    pub fn create(&mut self, path: PathBuf) -> Result<Entry, IOError>  {
        let mut reader = BufReader::new(
            try!(self.read_opener.get_reader(&path))
        );

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

    /// Creates and sends `Entry` instances on a
    pub fn send_many(&mut self, paths: Vec<PathBuf>, sender: Sender<Result<Entry, IOError>>)
                     -> Result<(), EntrySendError> {
        let mut unsendable = Vec::new();
        for path in paths.into_iter() {
            let entry = self.create(path);
            match sender.send(entry) {
                Ok(_) => {},
                Err(SendError(entry)) => {
                    match entry {
                        Ok(Entry {path: path, ..}) => unsendable.push(Ok(path)),
                        Err(err) => unsendable.push(Err(err)),
                    }
                }
            }
        }

        if unsendable.len() > 0 {
            Err(EntrySendError{ failed: unsendable })
        } else {
            Ok(())
        }
    }
}

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


#[cfg(test)]
mod test {
    use super::*;
    use hamcrest::prelude::*;
    use std::io::{Cursor, Read, Write};
    use std::io::Error as IOError;
    use std::path::PathBuf;
    use std::sync::mpsc::channel;
    use std::vec::Vec;

    use crypto::md5::Md5;
    use crypto::digest::Digest;

    struct CursorFactory;

    impl ReadOpener for CursorFactory {
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
    fn test_entry_factory_creates_entry() {
        let path = PathBuf::from("my/test/file-name");
        let expected = create_expected_entry(&path);

        let mut factory = EntryFactory::new(Md5::new(), CursorFactory);
        let entry = factory.create(path).unwrap();

        assert_that!(entry, is(equal_to(expected)));
    }

    #[test]
    fn test_entry_factory_resets_digest() {
        let path = PathBuf::from("my/test/file-name");
        let expected = create_expected_entry(&path);

        let mut factory = EntryFactory::new(Md5::new(), CursorFactory);

        factory.create(path.clone()).unwrap();
        // This second create will fail if initial_digest is not reset.
        let entry = factory.create(path).unwrap();
        assert_that!(entry, is(equal_to(expected)));
    }

    #[test]
    fn test_entry_factory_send_many() {
        let mut factory = EntryFactory::new(Md5::new(), CursorFactory);

        let paths = vec![PathBuf::from("a.jpg"),
                         PathBuf::from("b.jpg"),
                         PathBuf::from("c.jpg"),
                         PathBuf::from("d.jpg")];
        let (send, recv) = channel::<Result<Entry, IOError>>();

        let expected: Vec<Entry> = paths.iter()
                                        .map(|ref x| create_expected_entry(&x))
                                        .collect();

        assert!(factory.send_many(paths, send).is_ok());

        let mut entries: Vec<Entry> = Vec::with_capacity(4);
		loop {
			match recv.try_recv() {
				Ok(entry) => entries.push(entry.unwrap()),
				_ => break,
			}
        }

        assert_that!(entries, is(equal_to(expected)));
    }

    #[test]
    fn test_entry_factory_send_many_returns_all_failed_paths() {
        let mut factory = EntryFactory::new(Md5::new(), CursorFactory);

        let paths = vec![PathBuf::from("a.jpg"),
                         PathBuf::from("b.jpg"),
                         PathBuf::from("c.jpg"),
                         PathBuf::from("d.jpg")];
        let (send, recv) = channel::<Result<Entry, IOError>>();
        drop(recv);

        let err: EntrySendError = factory.send_many(paths.clone(), send).unwrap_err();
        let failed_paths: Vec<PathBuf> = err.failed.into_iter().map(|r| r.unwrap()).collect();

        assert_that!(failed_paths, is(equal_to(paths)));
    }

    fn create_expected_entry(path: &PathBuf) -> Entry {
        let mut buf = Vec::new();
        let mut reader = CursorFactory.get_reader(&path).unwrap();
        assert!(reader.read_to_end(&mut buf).is_ok());

        let mut md5 = Md5::new();
        md5.input(&mut buf[..]);

        Entry {path: path.clone(),
               hashes: vec![md5.result_str()]}
    }
}
