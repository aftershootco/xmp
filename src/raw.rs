use crate::*;
use std::io::BufReader;

#[derive(Debug, Clone, Copy, Default)]
pub struct Raw;

impl Image for Raw {}

impl Results<Raw> {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, XmpError> {
        Self::from_slice(BufReader::new(std::fs::File::open(path)?))
    }
}

impl UpdateResults<Raw> {
    pub fn update(&self, path: impl AsRef<Path>) -> Result<(), XmpError> {
        let xml = self
            .update_xml(BufReader::new(std::fs::File::open(&path)?))
            .unwrap_or_else(|_e| DEFAULT_XML.as_bytes().to_vec());
        let mut f = std::fs::File::create(path)?;
        let mut bf = BufWriter::new(&mut f);
        bf.write_all(xml.as_slice())?;

        Ok(())
    }
}

impl OptionalResults<Raw> {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, XmpError> {
        Self::from_slice(BufReader::new(std::fs::File::open(path)?))
    }
}

#[test]
pub fn xmp_file() {
    println!("{:?}", Results::<Raw>::load("assets/file.xmp").unwrap());
}

#[test]
pub fn set_xmp() {
    let x = UpdateResults {
        colors: Some(String::from("Blue")),
        ..Default::default()
    };
    UpdateResults::<Raw>::update(&x, "assets/f.xmp").unwrap();
    println!("{:?}", Results::<Raw>::load("assets/f.xmp").unwrap());
}
