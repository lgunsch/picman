use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::vec::Vec;

use entry::Entry;

pub struct DuplicationMap {
    map: HashMap<String, Vec<Entry>>,
}

impl DuplicationMap {
    pub fn new() -> DuplicationMap {
        let map = HashMap::new();
        DuplicationMap { map: map }
    }

    pub fn push(&mut self, entry: Entry) {
        let key = entry.primary_hash.clone();
        match self.map.entry(key) {
            Occupied(o) => o.into_mut().push(entry), // FIXME: Add a secondary hash here somehow
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
            None => return None, // FIXME: does this abandon some entries?
        };

        let mut curr: Vec<Entry> = Vec::new();
        for entry in entries {
            // FIXME: entries is not sorted, so it could be hash-1, hash-2, hash-1

            match curr.last().map(|e| e.clone()) {
                Some(e) => {
                    // it's a bug if the secondary_hash doesn't exist by now
                    if e.secondary_hash.as_ref().unwrap() == entry.secondary_hash.as_ref().unwrap()
                    {
                        curr.push(entry);
                    } else {
                        self.duplicates.push(curr);
                        curr = vec![entry];
                    }
                }
                None => curr.push(entry),
            };
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

    #[test]
    fn test_duplication_map_groups_duplicate_entries() {
        let mut map = DuplicationMap::new();
        let entry1 = add_entry(&mut map, "entry-1", "hash-1");
        let entry2 = add_entry(&mut map, "entry-2", "hash-1"); // duplicate

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
        let mut entry1 = Entry::new("entry-1", "hash-1");
        let mut entry2 = Entry::new("entry-2", "hash-1");

        entry1.secondary_hash = Some("hash-1-1".into());
        entry2.secondary_hash = Some("hash-1-2".into());

        map.push(entry1.clone());
        map.push(entry2.clone());

        let mut iter = map.into_iter();

        assert_that!(iter.next(), is(equal_to(Some(vec![entry1]))));
        assert_that!(iter.next(), is(equal_to(Some(vec![entry2]))));
    }

    fn add_entry(map: &mut DuplicationMap, path: &str, hash: &str) -> Entry {
        let entry = Entry::new(path, hash);
        map.push(entry.clone());
        return entry;
    }
}
