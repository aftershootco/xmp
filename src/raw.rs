use crate::*;
use std::io::Cursor;

pub(crate) fn __raw_load_xml(path: impl AsRef<Path>) -> Result<Vec<u8>, XmpError> {
    let mut processor = libraw_r::Processor::default();
    let exif = processor.set_exif_callback(
        Vec::<u8>::new(),
        libraw_r::exif::DataStreamType::File,
        |args| {
            if args.tag == 0x02bc {
                args.callback_data.extend_from_slice(args.data);
            }
            Ok(())
        },
    )?;
    processor.open(path)?;
    let xmp = exif.data()?;
    if !xmp.is_empty() {
        return Ok(xmp);
    }
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
