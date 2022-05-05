use crate::*;
use jfifdump::{Reader, SegmentKind};
use std::io::BufReader;

impl Results<Jpeg> {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, XmpError> {
        let file = std::fs::File::open(path)?;
        let bf = BufReader::new(file);
        let mut jfif = Reader::new(bf).unwrap();

        loop {
            let segment = jfif.next_segment().unwrap();
            match &segment.kind {
                SegmentKind::Eoi => break,
                SegmentKind::App { nr: 0x1, data } => {
                    // http://ns.adobe.com/xap/1.0
                    return Results::from_slice(data.as_slice());
                }
                _ => (),
            }
        }
        Err(XmpErrorKind::JFIFHeaderMissing.into())
    }
}

impl UpdateResults<Jpeg> {
    pub fn update(&self, path: impl AsRef<Path>) -> Result<(), XmpError> {
        
    }
}

#[test]
pub fn test_jpeg_load() {
    load("1.jpg").unwrap();
}
