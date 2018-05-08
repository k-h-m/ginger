extern crate walkdir;

use walkdir::WalkDir;

fn main() {
    let name = std::env::args().skip(1).next().unwrap();
    for entry in WalkDir::new(name) {
        let entry = entry.unwrap();
        let path = entry.path();
        let meta = std::fs::metadata(path).unwrap();
        if meta.file_type().is_file() {
            println!("{}", entry.path().display());
        }
    }
}
