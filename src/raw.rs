use crate::*;

#[derive(Debug, Clone, Copy, Default)]
pub struct Raw;

impl Image for Raw {}

impl Results<Raw> {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, XmpError> {
        let str = std::fs::read_to_string(path)?;
        Self::from_slice(str.as_bytes())
    }
}

impl UpdateResults<Raw> {
    pub fn update(&self, path: impl AsRef<Path>) -> Result<(), XmpError> {
        let xml = self
            .update_xml(std::fs::read_to_string(&path)?)
            .unwrap_or_else(|_e| DEFAULT_XML.to_string());
        let mut f = std::fs::File::create(path)?;
        let mut bf = BufWriter::new(&mut f);
        bf.write_all(xml.as_bytes())?;

        Ok(())
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
