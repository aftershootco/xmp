use std::path::{Path, PathBuf};
use xmp::*;
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<String>>();
    let argc = args.len();

    if argc < 2 {
        eprintln!("Give filename");
        std::process::exit(1);
    }
    let path = PathBuf::from(&args[1]);

    let r = OptionalResults::load(path)?;
    println!("{:#?}", r);
    Ok(())
}

pub fn is_jpeg(path: impl AsRef<Path>) -> Option<PathBuf> {
    let jpeg = path.as_ref().with_extension("jpg");
    if jpeg.exists() {
        std::fs::canonicalize(jpeg).ok()
    } else {
        None
    }
}
