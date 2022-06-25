use crate::*;
use std::io::Cursor;

pub(crate) fn __raw_load_xml(path: impl AsRef<Path>) -> Result<Vec<u8>, XmpError> {
    let mut processor = libraw_r::Processor::default();
    processor.open(path)?;
    let iparams = processor.iparams();

    let xmp = unsafe {
        std::slice::from_raw_parts(
            std::mem::transmute(iparams.xmpdata),
            iparams.xmplen as usize,
        )
    };
    Ok(xmp.to_vec())
}

impl UpdateResults {
    // pub fn update_raw(
    //     &self,
    //     path: impl AsRef<Path>,
    //     options: UpdateOptions,
    // ) -> Result<(), XmpError> {
    //     let xml = self
    //         .update_xml(BufReader::new(std::fs::File::open(&path)?), options)
    //         // .update_xml(std::fs::read(&path)?.as_slice(), false)
    //         .unwrap_or_else(|_e| DEFAULT_XML.as_bytes().to_vec());

    //     let mut bfw = BufWriter::new(std::fs::File::create(path)?);
    //     bfw.write_all(xml.as_slice())?;
    //     bfw.flush()?;

    //     Ok(())
    // }
}

impl OptionalResults {
    pub fn load_raw(path: impl AsRef<Path>) -> Result<Self, XmpError> {
        let data = Cursor::new(__raw_load_xml(&path)?);
        // std::fs::write("somefile.xmp", &__raw_load_xml(&path)?)?;
        Self::from_reader(data)
    }
}
