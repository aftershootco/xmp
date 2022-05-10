use crate::*;
use std::io::BufReader;


impl Results {
    pub fn load_raw(path: impl AsRef<Path>) -> Result<Self, XmpError> {
        Self::from_slice(BufReader::new(std::fs::File::open(path)?))
    }
}

impl UpdateResults {
    pub fn update_raw(&self, path: impl AsRef<Path>) -> Result<(), XmpError> {
        let xml = self
            .update_xml(BufReader::new(std::fs::File::open(&path)?), false)
            .unwrap_or_else(|_e| DEFAULT_XML.as_bytes().to_vec());
        let mut f = std::fs::File::create(path)?;
        let mut bf = BufWriter::new(&mut f);
        bf.write_all(xml.as_slice())?;

        Ok(())
    }
}

impl OptionalResults {
    pub fn load_raw(path: impl AsRef<Path>) -> Result<Self, XmpError> {
        Self::from_slice(BufReader::new(std::fs::File::open(path)?))
    }
}

#[test]
pub fn read_xmp() {
    println!("{:?}", OptionalResults::load("assets/file.xmp").unwrap());
}

#[test]
pub fn set_color() {
    let x = UpdateResults {
        colors: Some(String::from("Blue")),
        ..Default::default()
    };
    UpdateResults::update(&x, "assets/f.xmp").unwrap();
    println!("{:?}", Results::load("assets/f.xmp").unwrap());
}

#[test]
pub fn set_subjects() {
    let x = UpdateResults {
        subjects: Some(vec!["HELLO".to_owned(), "World".to_owned()]),
        hierarchies: Some(vec!["Some".to_owned(), "stuff".to_owned()]),
        ..Default::default()
    };
    UpdateResults::update(&x, "assets/f.xmp").unwrap();
    println!("{:?}", OptionalResults::load("assets/f.xmp").unwrap());
}
