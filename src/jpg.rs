use crate::*;
use exif::{Context, Field, In, Tag, Value};
use img_parts::ImageEXIF;
use jfifdump::{Reader, SegmentKind};
use std::io::BufReader;

impl Results {
    pub fn load_jpg(path: impl AsRef<Path>) -> Result<Self, XmpError> {
        let data = __jpeg_load_xml(path)?;
        Self::from_slice(data.as_slice())
    }
}

pub(crate) fn __jpeg_load_xml(path: impl AsRef<Path>) -> Result<Vec<u8>, XmpError> {
    // First open a buffered reader
    // Check if the file has a exif header or a jfif header
    // Its possible for files to have a exif header while also having a jfif header
    // So check for the exif header first
    let jpeg = img_parts::jpeg::Jpeg::from_bytes(std::fs::read(&path)?.into())?;
    if let Some(exif_data) = jpeg.exif() {
        let exifreader = exif::Reader::new();
        let exif = exifreader.read_raw(exif_data.to_vec())?;
        let __xmp_val = exif
            .get_field(Tag(Context::Exif, 700), In::PRIMARY)
            .map(|f| &f.value);
        if let Some(exif::Value::Undefined(data, _)) = __xmp_val {
            return Ok(data.to_vec());
        }
    }
    let file = std::fs::File::open(path)?;
    let bfr = BufReader::new(file);
    let mut jfif = Reader::new(bfr)?;

    loop {
        let segment = jfif.next_segment()?;
        match &segment.kind {
            SegmentKind::Eoi => break,
            SegmentKind::App { nr: 0x1, data } => {
                if data.starts_with(b"http://ns.adobe.com") {
                    return Ok(data.to_vec());
                }
            }
            _ => (),
        }
    }

    Err(XmpErrorKind::XMPMissing.into())
}

impl UpdateResults {
    pub fn update_jpg(&self, path: impl AsRef<Path>) -> Result<(), XmpError> {
        let xml = __jpeg_load_xml(&path).unwrap_or_else(|_e| DEFAULT_XML.as_bytes().to_vec());
        let xml = self.update_xml(BufReader::new(xml.as_slice()), false)?;
        let mut jpeg = img_parts::jpeg::Jpeg::from_bytes(std::fs::read(&path)?.into())?;
        // Do not overwrite the existing exif data
        let mut exifwriter = exif::experimental::Writer::new();

        let exif = jpeg.exif().and_then(|exif_data| {
            let exifreader = exif::Reader::new();
            exifreader.read_raw(exif_data.to_vec()).ok()
        });

        let exif_xml_tag = Field {
            tag: Tag(Context::Exif, 700),
            ifd_num: In::PRIMARY,
            value: Value::Undefined(xml, 0),
        };

        let mut exif_data = std::io::Cursor::new(Vec::new());

        if let Some(exif) = exif {
            exif.fields().for_each(|f| exifwriter.push_field(f));
            exifwriter.push_field(&exif_xml_tag);
            exifwriter.write(&mut exif_data, false)?;
        } else {
            exifwriter.push_field(&exif_xml_tag);
            exifwriter.write(&mut exif_data, false)?;
        }

        jpeg.remove_segments_by_marker(0xE0);
        jpeg.set_exif(Some(exif_data.into_inner().into()));

        let mut bfw = BufWriter::new(std::fs::File::create(path)?);

        jpeg.encoder().write_to(&mut bfw)?;
        bfw.flush()?;

        Ok(())
    }
}

impl OptionalResults {
    pub fn load_jpg(path: impl AsRef<Path>) -> Result<Self, XmpError> {
        let data = __jpeg_load_xml(path)?;
        Self::from_slice(data.as_slice())
    }
}

#[test]
pub fn test_jpeg_jfif_load() {
    Results::load("assets/1.jpg").unwrap();
}
#[test]
pub fn test_jpeg_exif_load() {
    Results::load("assets/2.jpg").unwrap();
}

#[test]
pub fn test_jpeg_exif_load_updated() {
    let x = UpdateResults {
        stars: Some(3),
        colors: Some(String::from("Blue")),
        ..Default::default()
    };
    println!("Writing");
    UpdateResults::update(&x, "assets/3.jpg").unwrap();
    UpdateResults::update(&x, "assets/4.jpg").unwrap();
    println!("Reading");
    println!("{:?}", Results::load("assets/3.jpg").unwrap());
}
