use std::error::Error;
use std::io::Error as IOError;
use std::fmt;
use std::path::PathBuf;
use std::sync::mpsc::{SendError, Sender};

use crypto::digest::Digest;
use utils::{HashDigester, ReadOpener};

/// Represents a single file, with its computed hash digest values
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entry {
    pub path: PathBuf,
    pub primary_hash: String,
    pub secondary_hash: Option<String>,
}

impl Entry {
    /// Creates a new `Entry` with the given path and primary_hash digest string.
    pub fn new<P, H>(path: P, primary_hash: H) -> Entry
    where
        P: Into<PathBuf>,
        H: Into<String>,
    {
        Entry {
            path: path.into(),
            primary_hash: primary_hash.into(),
            secondary_hash: None,
        }
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "primary-hash:{} second-hash: {} {}",
            self.primary_hash,
            self.secondary_hash.as_ref().unwrap_or(&"".to_string()),
            self.path.display()
        )
    }
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
pub struct EntryFactory<D, R>
where
    D: Digest,
    R: ReadOpener,
{
    primary_digester: HashDigester<D, R>,
}

impl<D, R> EntryFactory<D, R>
where
    D: Digest,
    R: ReadOpener,
{
    /// Creates an `EntryFactory` using the initial `Digest`, and `ReadOpener`.
    pub fn new(digester: HashDigester<D, R>) -> EntryFactory<D, R> {
        EntryFactory {
            primary_digester: digester,
        }
    }

    /// Creates an `Entry`, also computing its first digest.
    pub fn create(&mut self, path: PathBuf) -> Result<Entry, IOError> {
        let primary_hash = self.primary_digester.get_digest(&path)?;
        Ok(Entry::new(path, primary_hash))
    }

    /// Creates and sends `Entry` instances on a
    pub fn send_many(
        &mut self,
        paths: Vec<PathBuf>,
        sender: &Sender<Result<Entry, IOError>>,
    ) -> Result<(), EntrySendError> {
        let mut unsendable = Vec::new();
        for path in paths.into_iter() {
            let entry = self.create(path);
            match sender.send(entry) {
                Ok(_) => {}
                Err(SendError(entry)) => match entry {
                    Ok(Entry { path, .. }) => unsendable.push(Ok(path)),
                    Err(err) => unsendable.push(Err(err)),
                },
            }
        }

        if unsendable.len() > 0 {
            Err(EntrySendError { failed: unsendable })
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use hamcrest::prelude::*;
    use std::io::{Cursor, Write};
    use std::io::Error as IOError;
    use std::path::PathBuf;
    use std::sync::mpsc::channel;
    use std::vec::Vec;

    use crypto::md5::Md5;

    use utils::{CursorReadOpener, HashDigester};

    #[test]
    fn test_entry_factory_creates_entry() {
        let path = PathBuf::from("my/test/file-name");
        let mut digester = create_digester(&path);
        let primary = digester.get_digest(&path).unwrap();
        let expected = Entry::new(&path, primary);

        let mut factory = EntryFactory::new(digester);
        let entry = factory.create(path).unwrap();

        assert_that!(entry, is(equal_to(expected)));
    }

    #[test]
    fn test_entry_factory_send_many() {
        let mut opener = CursorReadOpener::new();

        let paths = vec![
            PathBuf::from("a.jpg"),
            PathBuf::from("b.jpg"),
            PathBuf::from("c.jpg"),
            PathBuf::from("d.jpg"),
        ];

        for path in &paths {
            opener.add_path(&path, create_cursor(&path));
        }

        let mut digester = HashDigester::new(Md5::new(), opener);
        let (send, recv) = channel::<Result<Entry, IOError>>();

        let expected: Vec<Entry> = paths
            .iter()
            .map(|ref x| Entry::new(&x, digester.get_digest(&x).unwrap()))
            .collect();

        let mut factory = EntryFactory::new(digester);

        assert!(factory.send_many(paths, &send).is_ok());

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
        let mut opener = CursorReadOpener::new();
        let paths = vec![
            PathBuf::from("a.jpg"),
            PathBuf::from("b.jpg"),
            PathBuf::from("c.jpg"),
            PathBuf::from("d.jpg"),
        ];

        for path in &paths {
            opener.add_path(&path, create_cursor(&path));
        }

        let (send, recv) = channel::<Result<Entry, IOError>>();
        drop(recv);

        let digester = HashDigester::new(Md5::new(), opener);
        let mut factory = EntryFactory::new(digester);

        let err: EntrySendError = factory.send_many(paths.clone(), &send).unwrap_err();
        let failed_paths: Vec<PathBuf> = err.failed.into_iter().map(|r| r.unwrap()).collect();

        assert_that!(failed_paths, is(equal_to(paths)));
    }

    /// Create a `HashDigester` using path &str as the file contents for
    /// `CursorReadOpener`
    fn create_digester(path: &PathBuf) -> HashDigester<Md5, CursorReadOpener> {
        let mut opener = CursorReadOpener::new();
        opener.add_path(&path, create_cursor(&path));
        return HashDigester::new(Md5::new(), opener);
    }

    /// Create a `Cursor` using path &str as the file contents for testing
    fn create_cursor(path: &PathBuf) -> Cursor<Vec<u8>> {
        let mut cursor = Cursor::new(Vec::new());
        let data: &[u8] = path.as_path().to_str().unwrap().as_bytes();
        assert_that!(cursor.write(&data).unwrap(), is(equal_to(data.len())));
        return cursor;
    }
}
