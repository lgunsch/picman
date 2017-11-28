use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::vec::Vec;

use entry::{Entry};

pub struct DuplicationMap {
    map: HashMap<String, Vec<Entry>>,
}

impl DuplicationMap {
    pub fn new() -> DuplicationMap {
        let map = HashMap::new();
        DuplicationMap {map: map}
    }

    pub fn push(&mut self, entry: Entry) {
        let key = entry.hashes[0].clone();
        match self.map.entry(key) {
            Occupied(o) => o.into_mut().push(entry),
            Vacant(v) => v.insert(Vec::new()).push(entry),
        };
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
            None => return None,
        };

        let mut curr: Vec<Entry> = Vec::new();
        for entry in entries {

            // try and_Then here

            if curr.last().map_or(false, |e| {
                if e.hashes[1] == entry.hashes[1] {
                    curr.push(entry);
                } else {
                    self.duplicates.push(curr);
                    curr = vec![entry];
                }
                return true;
            }) {
                curr.push(entry);
            }
        //     let last = curr.last();
        //     match last {
        //         Some(e) => {
        //             if e.hashes[1] == entry.hashes[1] {
        //                 curr.push(entry);
        //             } else {
        //                 self.duplicates.push(curr);
        //                 curr = vec![entry];
        //             }
        //         },
        //         None => curr.push(entry),
        //     };
        // }
        return self.duplicates.pop();
    }
}

impl IntoIterator for DuplicationMap {
    type Item = Vec<Entry>;
    type IntoIter = DuplicationMapIterator;

    fn into_iter(self) -> Self::IntoIter {
        DuplicationMapIterator { map_iter: self.map.into_iter(),
                                 duplicates: Vec::new() }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;
    use hamcrest::prelude::*;

    #[test]
    fn test_duplication_map_groups_duplicate_entries() {
        let mut map = DuplicationMap::new();
        let entry1 = add_entry(&mut map, "entry-1", "hash-1");
        let entry2 = add_entry(&mut map, "entry-2", "hash-1");  // duplicate

        let mut iter = map.into_iter();

        let expected = Some(vec![entry1, entry2]);

        assert_that!(iter.next(), is(equal_to(expected)));
    }

    #[test]
    fn test_duplication_map_splits_unique_entries() {
        let mut map = DuplicationMap::new();
        add_entry(&mut map, "entry-1", "hash-1");
        add_entry(&mut map, "entry-2", "hash-2");

        let mut iter = map.into_iter();

        assert_that!(iter.next().unwrap().len(), is(equal_to(1)));
        assert_that!(iter.next().unwrap().len(), is(equal_to(1)));
    }

    #[test]
    fn test_duplication_map_splits_second_hash_unique() {
        let mut map = DuplicationMap::new();
        let mut entry1 = make_entry("entry-1", "hash-1");
        let mut entry2 = make_entry("entry-2", "hash-1");

        entry1.hashes.push("hash-1-1".into());
        entry2.hashes.push("hash-1-2".into());

        map.push(entry1.clone());
        map.push(entry2.clone());

        let mut iter = map.into_iter();

        assert_that!(iter.next(), is(equal_to(Some(vec![entry1]))));
        assert_that!(iter.next(), is(equal_to(Some(vec![entry2]))));
    }

    fn add_entry<I: Into<String>>(map: &mut DuplicationMap, path: I, hash: I) -> Entry {
        let entry = make_entry(path, hash);
        map.push(entry.clone());
        return entry;
    }

    fn make_entry<I: Into<String>>(path: I, hash: I) -> Entry {
        let mut entry = Entry::new(PathBuf::from(path.into()));
        entry.hashes = vec![hash.into()];
        return entry;
    }
}
