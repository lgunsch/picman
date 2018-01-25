/// Entry integration tests
use std::path::PathBuf;

use crypto::md5::Md5;
use hamcrest::prelude::*;

use entry::{Entry, EntryFactory};
use utils::{FileReadOpener, HashDigester};

#[test]
fn entry_factory_creates_entry() {
    let digester = HashDigester::new(Md5::new(), FileReadOpener::new());
    let mut factory = EntryFactory::new(digester);
    let path = PathBuf::from("./src/tests/assets/barbara.png");
    let entry = factory.create(path.clone()).unwrap();

    // hash computed externally using md5sum
    let expected = Entry::new(path, "73c4b3758af64736831438b028ac4524");
    assert_that!(entry, is(equal_to(expected)));
}
