use crate::*;
// use std::io::BufReader;

impl Results {
    pub fn load_raw(path: impl AsRef<Path>) -> Result<Self, XmpError> {
        Self::from_reader(BufReader::new(std::fs::File::open(path)?))
        // let data = std::fs::read(path)?;
        // Self::from_reader(data)
    }
}

impl UpdateResults {
    pub fn update_raw(&self, path: impl AsRef<Path>) -> Result<(), XmpError> {
        let xml = self
            .update_xml(BufReader::new(std::fs::File::open(&path)?), false)
            // .update_xml(std::fs::read(&path)?.as_slice(), false)
            .unwrap_or_else(|_e| DEFAULT_XML.as_bytes().to_vec());

        let mut bfw = BufWriter::new(std::fs::File::create(path)?);
        bfw.write_all(xml.as_slice())?;
        bfw.flush()?;

        Ok(())
    }
}

impl OptionalResults {
    pub fn load_raw(path: impl AsRef<Path>) -> Result<Self, XmpError> {
        Self::from_reader(BufReader::new(std::fs::File::open(path)?))
        // let data = std::fs::read(path)?;
        // Self::from_reader(data.as_slice())
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

#[test]
pub fn missing_namespace() {
    let e = OptionalResults {
        stars: Some(3),
        datetime: Some(1651333056),
        ..Default::default()
    };
    let r = OptionalResults::load("assets/missing_ns.xmp").unwrap();
    assert_eq!(e, r);
}
