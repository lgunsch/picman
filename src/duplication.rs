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
        // remove from duplicates first
        let entries: Vec<Entry> = match self.map_iter.next() {
            Some((_k, v)) => v,
            None => return None, // FIXME: does this abandon some entries?
        };

        let mut curr: Vec<Entry> = Vec::new();
        for entry in entries {
            // FIXME: entries is not sorted, so it could be hash-1, hash-2, hash-1

            match curr.last().cloned() {
                Some(last) => {
                    // it's a bug if the secondary_hash doesn't exist by now
                    if last.is_duplicate(&entry) {
                        curr.push(entry);
                    } else {
                        // save the previous duplicate list,
                        // get ready for the next list
                        self.duplicates.push(curr);
                        curr = vec![entry];
                    }
                }
                None => curr.push(entry),
            };
        }
        if self.duplicates.is_empty() {
            return Some(curr);
        } else {
            return self.duplicates.pop()  // FIXME: if this is not empty, pop before doing anything
        }
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
    fn test_duplication_map_groups_primary_hash_duplicate_entries() {
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

    // #[test]
    // fn test_duplication_map_splits_unique_entries() {
    //     let mut map = DuplicationMap::new();
    //     add_entry(&mut map, "entry-1", "hash-1");
    //     add_entry(&mut map, "entry-2", "hash-2");

    //     let mut iter = map.into_iter();

    //     assert_that!(iter.next().unwrap().len(), is(equal_to(1)));
    //     assert_that!(iter.next().unwrap().len(), is(equal_to(1)));
    // }

    // #[test]
    // fn test_duplication_map_splits_second_hash_unique() {
    //     let mut map = DuplicationMap::new();
    //     let mut entry1 = Entry::new("entry-1", "hash-1");
    //     let mut entry2 = Entry::new("entry-2", "hash-1");

    //     entry1.secondary_hash = Some("hash-1-1".into());
    //     entry2.secondary_hash = Some("hash-1-2".into());

    //     map.push(entry1.clone());
    //     map.push(entry2.clone());

    //     let mut iter = map.into_iter();

    //     assert_that!(iter.next(), is(equal_to(Some(vec![entry1]))));
    //     assert_that!(iter.next(), is(equal_to(Some(vec![entry2]))));
    // }

    // fn add_entry<D: Digester>(map: &mut DuplicationMap<D>, path: &str, hash: &str) -> Entry {
    //     let entry = Entry::new(path, hash);
    //     map.push(entry.clone());
    //     return entry;
    // }

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
