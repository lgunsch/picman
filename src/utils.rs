use std::io::Read;
use std::io::Error as IOError;
use std::fs::File;
use std::path::PathBuf;

pub trait ReadOpener {
    type Readable: Read;
    /// Creates a new `Readable` type, given a `PathBuf`.
    fn get_reader(&mut self, &PathBuf) -> Result<Self::Readable, IOError>;
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
