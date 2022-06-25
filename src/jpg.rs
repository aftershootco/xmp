use crate::*;
use exif::{Context, Field, In, Tag, Value};
use img_parts::ImageEXIF;
use jfifdump::{Reader, SegmentKind};
use std::io::BufReader;
use std::io::Cursor;

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
    pub fn update_jpg(
        &self,
        path: impl AsRef<Path>,
        options: UpdateOptions,
    ) -> Result<(), XmpError> {
        let xml = __jpeg_load_xml(&path).unwrap_or_else(|_e| DEFAULT_XML.as_bytes().to_vec());
        // let xml = self.update_xml(Cursor::new(xml))?;
        let xml = self.update_xml(Cursor::new(xml), options)?;
        let data = std::fs::read(&path)?;
        // let mut jpeg = img_parts::jpeg::Jpeg::from_bytes(std::fs::read(&path)?.into())?;
        let mut jpeg = img_parts::jpeg::Jpeg::from_bytes(data.into())
            .map_err(|e| XmpError::from(e).with_name(path.as_ref()))?;
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
            let mut insert: HashSet<Tag> = exif.fields().map(|f| f.tag).collect();
            for field in exif.fields() {
                if insert.contains(&field.tag) {
                    exifwriter.push_field(field);
                    insert.remove(&field.tag);
                }
            }
            exifwriter.push_field(&exif_xml_tag);
            exifwriter
                .write(&mut exif_data, false)
                .map_err(|e| XmpError::from(e).with_name(path.as_ref()))?;
        } else {
            exifwriter.push_field(&exif_xml_tag);
            exifwriter
                .write(&mut exif_data, false)
                .map_err(|e| XmpError::from(e).with_name(path.as_ref()))?;
        }

        let exif_data = exif_data.into_inner();
        jpeg.remove_segments_by_marker(0xE0);
        jpeg.set_exif(Some(exif_data.into()));

        let temp = path.as_ref().with_extension("temp");
        let mut bfw = BufWriter::new(std::fs::File::create(&temp)?);

        // This might cause panics
        jpeg.encoder()
            .write_to(&mut bfw)
            .map_err(|e| XmpError::from(e).with_name(path.as_ref()))?;
        bfw.flush()?;
        std::fs::rename(temp, path)?;

        Ok(())
    }
}

impl OptionalResults {
    pub fn load_jpg(path: impl AsRef<Path>) -> Result<Self, XmpError> {
        let data_cursor = Cursor::new(__jpeg_load_xml(path)?);
        Self::from_reader(data_cursor)
    }
}

// #[test]
// pub fn test_jpeg_jfif_load() {
//     let r = OptionalResults::load("assets/1.jpg").unwrap();
//     let e = Results {
//         stars: 5,
//         colors: "Red".to_string(),
//         datetime: 1633790597,
//         subjects: vec!["Duplicates", "Selected"]
//             .iter()
//             .map(ToString::to_string)
//             .collect(),
//         hierarchies: vec!["Duplicates"].iter().map(ToString::to_string).collect(),
//     };
//     assert_eq!(r, e);
// }
// #[test]
// pub fn test_jpeg_exif_load() {
//     let e = Results {
//         stars: 3,
//         colors: "Yellow".to_string(),
//         datetime: 0,
//         subjects: vec!["Duplicates", "Selected"]
//             .iter()
//             .map(ToString::to_string)
//             .collect(),
//         hierarchies: vec!["Duplicates"].iter().map(ToString::to_string).collect(),
//     };
//     let r = Results::load("assets/2.jpg").unwrap();
//     assert_eq!(r, e);
// }

#[test]
pub fn test_jpeg_exif_load_updated() {
    let r_1 = OptionalResults::load("assets/3.jpg").unwrap();
    let u = UpdateResults {
        stars: Some(3),
        colors: Some(String::from("Blue")),
        orientation: Some(1),
        ..Default::default()
    };
    u.update("assets/3.jpg").unwrap();
    let r_2 = OptionalResults::load("assets/3.jpg").unwrap();
    let e = OptionalResults {
        stars: Some(3),
        colors: Some(String::from("Blue")),
        datetime: Some(1690139150),
        subjects: Some(
            vec!["Duplicates", "Selected"]
                .iter()
                .map(ToString::to_string)
                .collect(),
        ),
        orientation: None,
        hierarchies: Some(vec!["Duplicates"].iter().map(ToString::to_string).collect()),
    };
    assert_eq!(r_2, e);

    let u = UpdateResults {
        stars: Some(5),
        colors: Some(String::from("Red")),
        ..Default::default()
    };
    u.update("assets/3.jpg").unwrap();
    let r_3 = OptionalResults::load("assets/3.jpg").unwrap();
    assert_eq!(r_1, r_3);
}
