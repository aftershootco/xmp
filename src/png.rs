use crate::*;
use exif::{Context, In, Tag};
use std::io::Cursor;

pub(crate) fn __png_load_xml(path: impl AsRef<Path>) -> Result<Vec<u8>, XmpError> {
    use img_parts::ImageEXIF;
    // First open a buffered reader
    // Check if the file has a exif header or a jfif header
    // Its possible for files to have a exif header while also having a jfif header
    // So check for the exif header first
    let png = img_parts::png::Png::from_bytes(std::fs::read(&path)?.into())?;

    if let Some(exif_data) = png.exif() {
        let exifreader = exif::Reader::new();
        let exif = exifreader.read_raw(exif_data.to_vec())?;
        let __xmp_val = exif
            .get_field(Tag(Context::Exif, 700), In::PRIMARY)
            .map(|f| &f.value);
        if let Some(exif::Value::Undefined(data, _)) = __xmp_val {
            return Ok(data.to_vec());
        }
    }

    let xml = png
        .chunk_by_type(*b"iTXt")
        .ok_or_else(|| XmpError::from(XmpErrorKind::XMPMissing))?;

    Ok(xml.contents().to_vec())
}

impl UpdateResults {
    pub fn update_png(
        &self,
        path: impl AsRef<Path>,
        options: UpdateOptions,
    ) -> Result<(), XmpError> {
        let xml = __png_load_xml(&path).unwrap_or_else(|_e| DEFAULT_XML.as_bytes().to_vec());
        // let xml = self.update_xml(Cursor::new(xml))?;
        let xml = self.update_xml(Cursor::new(xml), options)?;

        let mut png = img_parts::png::Png::from_bytes(std::fs::read(&path)?.into())?;

        for chunk in png.chunks_mut() {
            if &chunk.kind() == b"iTXt" {
                *chunk = img_parts::png::PngChunk::new(*b"iTXt", xml.into());
                break;
            }
        }

        Ok(())
    }
}

impl OptionalResults {
    pub fn load_png(path: impl AsRef<Path>) -> Result<Self, XmpError> {
        let data = Cursor::new(__png_load_xml(path)?);
        Self::from_reader(data)
    }
}

impl Results {
    pub fn load_png(path: impl AsRef<Path>) -> Result<Self, XmpError> {
        let data = Cursor::new(__png_load_xml(path)?);
        Self::from_reader(data)
    }
}
