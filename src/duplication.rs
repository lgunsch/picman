use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::io::Error as IOError;
use std::vec::Vec;

use entry::Entry;
use utils::{Digester};

#[derive(Default)]
pub struct DuplicationMap {
    map: HashMap<String, Vec<Entry>>,
}

impl DuplicationMap {
    pub fn new() -> DuplicationMap {
        let map = HashMap::new();
        DuplicationMap { map: map }
    }

    pub fn push<D: Digester>(&mut self, secondary_digester: &mut D, entry: Entry)
                             -> Result<(), IOError>
    {
        let key = entry.primary_hash.clone();
        match self.map.entry(key) {
            Occupied(o) => {
                let duplicates = o.into_mut();
                duplicates.push(entry);

                for entry in duplicates {
                    match entry.secondary_hash {
                        Some(_) => continue,
                        None => {
                            let mut hash = secondary_digester.get_digest(&entry.path)?;
                            entry.secondary_hash = Some(hash);
                        },
                    }
                }
            },
            Vacant(v) => v.insert(Vec::new()).push(entry),
        };

        Ok(())
    }
}

pub struct DuplicationMapIterator {
    map_iter: <HashMap<String, Vec<Entry>> as IntoIterator>::IntoIter,
    duplicates: Vec<Vec<Entry>>,
}

impl Iterator for DuplicationMapIterator {
    type Item = Vec<Entry>;

    fn next(&mut self) -> Option<Vec<Entry>> {
        // return any leftovers from previous run first
        if !self.duplicates.is_empty() {
            return self.duplicates.pop();
        }

        let possible_duplicates: Vec<Entry> = match self.map_iter.next() {
            Some((_k, v)) => v,
            None => return None,
        };

        // secondary_hash will not exist if there is only a single entry
        if possible_duplicates.len() == 1 {
            return Some(possible_duplicates);
        }

        // find duplicates by grouping entries on secondary_hash
        let mut unique_entries: HashMap<String, Vec<Entry>> = HashMap::new();
        for entry in possible_duplicates {
            let key = entry.secondary_hash.clone()
                .expect("secondary_hash must exist for duplicated primary_hash");
            let mut dups = unique_entries.entry(key)
                .or_insert_with(|| Vec::new());
            dups.push(entry);
        }

        for (_, entries) in unique_entries {
            self.duplicates.push(entries)
        }

        return self.duplicates.pop();
    }
}

impl IntoIterator for DuplicationMap {
    type Item = Vec<Entry>;
    type IntoIter = DuplicationMapIterator;

    fn into_iter(self) -> Self::IntoIter {
        DuplicationMapIterator {
            map_iter: self.map.into_iter(),
            duplicates: Vec::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use hamcrest::prelude::*;

    use std::io::{Cursor, Error as IOError, ErrorKind, Write};
    use std::path::PathBuf;

    use utils::CursorReadOpener;

    struct StubDigester {
        map: HashMap<PathBuf, Result<String, IOError>>
    }

    impl StubDigester {
        fn new() -> StubDigester {
            StubDigester { map: HashMap::new() }
        }

        fn add_path_digest<P>(&mut self, p: P, result: Result<String, IOError>)
        where
            P: Into<PathBuf>,
        {
            self.map.insert(p.into(), result);
        }
    }

    impl Digester for StubDigester {
        fn get_digest(&mut self, path: &PathBuf) -> Result<String, IOError> {
            return match self.map.remove(path) {
                Some(v) => v,
                None => {
                    let msg = format!("digest not found {}", path.as_path().display());
                    Err(IOError::new(ErrorKind::NotConnected, msg))
                },
            }
        }
    }

    #[test]
    fn test_duplication_map_groups_duplicate_entries() {
        let mut opener = CursorReadOpener::new();
        opener.add_path("entry-1", create_cursor("hash-1"));
        opener.add_path("entry-2", create_cursor("hash-1"));

        let mut digester = StubDigester::new();
        digester.add_path_digest("entry-1", Ok("second-hash-1".to_owned()));
        digester.add_path_digest("entry-2", Ok("second-hash-1".to_owned()));

        let mut map = DuplicationMap::new();
        assert!(map.push(&mut digester, Entry::new("entry-1", "hash-1")).is_ok());
        assert!(map.push(&mut digester, Entry::new("entry-2", "hash-1")).is_ok());

        let mut iter = map.into_iter();

        let expected = Some(vec![entry("entry-1", "hash-1", "second-hash-1"),
                                 entry("entry-2", "hash-1", "second-hash-1")]);

        assert_that!(iter.next(), is(equal_to(expected)));
    }

    #[test]
    fn test_duplication_map_splits_unique_entries() {
        let mut opener = CursorReadOpener::new();
        opener.add_path("entry-1", create_cursor("hash-1"));
        opener.add_path("entry-2", create_cursor("hash-2"));

        let mut secondary_digester = StubDigester::new();
        secondary_digester.add_path_digest("entry-1", Ok("second-hash-1".to_owned()));
        secondary_digester.add_path_digest("entry-2", Ok("second-hash-2".to_owned()));

        let mut map = DuplicationMap::new();
        assert!(map.push(&mut secondary_digester, Entry::new("entry-1", "hash-1")).is_ok());
        assert!(map.push(&mut secondary_digester, Entry::new("entry-2", "hash-2")).is_ok());

        let mut iter = map.into_iter();

        assert_that!(iter.next().unwrap().len(), is(equal_to(1)));
        assert_that!(iter.next().unwrap().len(), is(equal_to(1)));
    }

    #[test]
    fn test_duplication_map_splits_second_hash_unique() {
        let mut secondary_digester = StubDigester::new();
        secondary_digester.add_path_digest("entry-1", Ok("second-hash-1".to_owned()));
        secondary_digester.add_path_digest("entry-2", Ok("second-hash-2".to_owned()));

        let mut map = DuplicationMap::new();
        assert!(map.push(&mut secondary_digester, Entry::new("entry-1", "hash-1")).is_ok());
        assert!(map.push(&mut secondary_digester, Entry::new("entry-2", "hash-1")).is_ok());

        let mut iter = map.into_iter();

        assert_that!(iter.next().unwrap().len(), is(equal_to(1)));
        assert_that!(iter.next().unwrap().len(), is(equal_to(1)));
    }

    #[test]
    fn test_duplication_map_groups_duplicate_entries_sorted() {
        let mut opener = CursorReadOpener::new();
        opener.add_path("entry-1", create_cursor("hash-1"));
        opener.add_path("entry-2", create_cursor("hash-1"));
        opener.add_path("entry-3", create_cursor("hash-1"));

        let mut digester = StubDigester::new();
        digester.add_path_digest("entry-1", Ok("second-hash-1".to_owned()));
        digester.add_path_digest("entry-2", Ok("second-hash-2".to_owned()));
        digester.add_path_digest("entry-3", Ok("second-hash-1".to_owned()));

        let mut map = DuplicationMap::new();
        assert!(map.push(&mut digester, Entry::new("entry-1", "hash-1")).is_ok());
        assert!(map.push(&mut digester, Entry::new("entry-2", "hash-1")).is_ok());
        assert!(map.push(&mut digester, Entry::new("entry-3", "hash-1")).is_ok());

        let mut iter = map.into_iter();

        let expected_a = vec![entry("entry-1", "hash-1", "second-hash-1"),
                              entry("entry-3", "hash-1", "second-hash-1")];
        let expected_b = vec![entry("entry-2", "hash-1", "second-hash-2")];

        assert_that!(iter.next().unwrap(), any_of!(equal_to(expected_a.clone()), equal_to(expected_b.clone())));
        assert_that!(iter.next().unwrap(), any_of!(equal_to(expected_a), equal_to(expected_b)));
    }

    fn create_cursor(hash: &str) -> Cursor<Vec<u8>> {
        // use path &str as file contents for testing
        let mut cursor = Cursor::new(Vec::new());
        let data: &[u8] = hash.as_bytes();
        assert_that!(cursor.write(&data).unwrap(), is(equal_to(data.len())));
        return cursor;
    }

    fn entry(path: &str, primary_hash: &str, secondary_hash: &str) -> Entry {
        let mut entry = Entry::new(path, primary_hash);
        entry.secondary_hash =  Some(secondary_hash.to_owned());
        return entry;
    }
}
