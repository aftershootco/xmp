use crate::*;
use std::io::Cursor;

pub(crate) fn __raw_load_xml(path: impl AsRef<Path>) -> Result<Vec<u8>, XmpError> {
    let mut processor = libraw_r::Processor::default();
    processor.open(path)?;
    let xmp = processor.xmpdata()?;
    Ok(xmp.to_vec())
}

impl OptionalResults {
    pub fn load_raw(path: impl AsRef<Path>) -> Result<Self, XmpError> {
        let data = Cursor::new(__raw_load_xml(&path)?);
        // std::fs::write("somefile.xmp", &__raw_load_xml(&path)?)?;
        Self::from_reader(data)
    }
}
