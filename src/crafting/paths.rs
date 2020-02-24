use std::path::Path;
use std::fs::File;

pub fn open_data(filename: &str) -> impl std::io::Read {
    File::open(Path::new("data").join(filename)).unwrap()
}
