use exif::{In, Tag};
use libraw_r::LibrawConstructorFlags;

use crate::errors::Result;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// See [here](https://jdhao.github.io/2019/07/31/image_rotation_exif_info/)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Orientation(pub u8);
impl Default for Orientation {
    fn default() -> Self {
        Self(1) // No rotation
    }
}

impl std::ops::Add for Orientation {
    type Output = Self;
    fn add(self, rhs: Orientation) -> Self::Output {
        Self(match (self.0, rhs.0) {
            (1, o) => o,
            (o, 1) => o,

            (2, 2) => 1,
            (2, 3) => 4,
            (2, 4) => 3,
            (2, 5) => 6,
            (2, 6) => 5,
            (2, 7) => 8,
            (2, 8) => 7,

            (3, 2) => 4,
            (3, 3) => 1,
            (3, 4) => 2,
            (3, 5) => 7,
            (3, 6) => 8,
            (3, 7) => 5,
            (3, 8) => 6,

            (4, 2) => 3,
            (4, 3) => 2,
            (4, 4) => 1,
            (4, 5) => 8,
            (4, 6) => 7,
            (4, 7) => 6,
            (4, 8) => 5,

            (5, 2) => 8,
            (5, 3) => 7,
            (5, 4) => 6,
            (5, 5) => 1,
            (5, 6) => 4,
            (5, 7) => 3,
            (5, 8) => 2,

            (6, 2) => 7,
            (6, 3) => 8,
            (6, 4) => 5,
            (6, 5) => 2,
            (6, 6) => 3,
            (6, 7) => 4,
            (6, 8) => 1,

            (7, 2) => 6,
            (7, 3) => 5,
            (7, 4) => 8,
            (7, 5) => 3,
            (7, 6) => 2,
            (7, 7) => 1,
            (7, 8) => 4,

            (8, 2) => 5,
            (8, 3) => 6,
            (8, 4) => 7,
            (8, 5) => 4,
            (8, 6) => 1,
            (8, 7) => 2,
            (8, 8) => 3,

            (_, _) => 1,
        })
    }
}

impl std::ops::Neg for Orientation {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(match self.0 {
            1 => 1,
            2 => 2,
            3 => 3,
            4 => 4,
            5 => 5,
            6 => 8,
            7 => 7,
            8 => 6,
            o => o,
        })
    }
}

impl PartialEq<u8> for Orientation {
    fn eq(&self, other: &u8) -> bool {
        &self.0 == other
    }
}
impl PartialEq<Orientation> for u8 {
    fn eq(&self, other: &Orientation) -> bool {
        self == &other.0
    }
}

impl Orientation {
    pub const fn new(o: u8) -> Option<Self> {
        if o > 8 {
            None
        } else {
            Some(Self(o))
        }
    }

    /// Reads the flip from raw converts it to exif::Tag::Orientation and then writes that
    /// to the jpeg buffer
    pub fn add_to_buffer_from_raw(buffer: &mut Vec<u8>, path: impl AsRef<Path>) -> Result<()> {
        let o = Self::from_raw(path)?;
        if o != 1 {
            o.add_exif(buffer)
        } else {
            Ok(())
        }
    }

    /// Add exif data with orientation to a file
    pub fn write_exif(&self, path: impl AsRef<Path>) -> Result<()> {
        use img_parts::ImageEXIF;
        let orientation = self.0;

        if orientation > 8 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Flip greater than 8",
            ))?;
        }
        let input = std::fs::read(&path)?;
        let mut jpeg = img_parts::jpeg::Jpeg::from_bytes(input.into())?;
        jpeg.set_exif(Some(Self::exif_data_with_orientation(orientation).into()));
        jpeg.encoder().write_to(std::fs::File::open(path)?)?;
        Ok(())
    }

    /// Add exif data with orientation to a buffer
    pub fn add_exif(&self, buffer: &mut Vec<u8>) -> Result<()> {
        use img_parts::ImageEXIF;
        let orientation = self.0;
        if orientation > 8 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Flip greater than 8",
            ))?;
        }
        let mut jpeg =
            img_parts::jpeg::Jpeg::from_bytes(img_parts::Bytes::from_iter(buffer.drain(..)))?;
        jpeg.set_exif(Some(Self::exif_data_with_orientation(orientation).into()));
        jpeg.encoder().write_to(buffer)?;

        Ok(())
    }

    /// Returns the raw flip / orientation value
    pub fn from_exif(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(&path)?;
        let mut bufreader = BufReader::new(&file);
        let exif = exif::Reader::new().read_from_container(&mut bufreader)?;

        if let Some(orientation) = exif.get_field(Tag::Orientation, In::PRIMARY) {
            Ok(Self(orientation.value.get_uint(0).unwrap_or(1) as u8))
        } else {
            // Ok(Self::from_flip(0))
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No Orientation tag in file",
            ))?
        }
    }

    /// Returns the flip value from a raw flie
    ///
    /// Check
    /// https://www.libraw.org/node/2445
    /// tl;dr libraw imgdata.sizes.flip and tiff:Orientation tag are different.
    pub fn from_raw(path: impl AsRef<Path>) -> Result<Self> {
        let mut processor = libraw_r::Processor::new(LibrawConstructorFlags::NoDataErrCallBack);
        processor.open(&path)?;

        // libraw imgdata.sizes.flip and tiff:Orientation tag are different.
        Ok(Self(Self::libraw_flip_to_orientation(
            processor.sizes().flip,
        )))
    }

    /// Converts libraw imgdata.sizes.flip to exif::Tag::Orientation
    pub fn libraw_flip_to_orientation(flip: i32) -> u8 {
        // libraw imgdata.sizes.flip and tiff:Orientation tag are different.
        match flip {
            0 => 1, // No rotation
            3 => 3, // To add to the confusion 3 is rotate 180 in both
            5 => 8, // Rotate 270 CW or 90 CCW
            6 => 6, // 6 is rotate 90 CW in both because yes
            _ => 1, // Invalid value so no rotation
        }
    }

    /// This encodes the flip into a raw exif container data
    fn exif_data_with_orientation(o: u8) -> Vec<u8> {
        vec![
            0x4d, 0x4d, 0x0, 0x2a, 0x0, 0x0, 0x0, 0x8, 0x0, 0x1, 0x1, 0x12, 0x0, 0x3, 0x0, 0x0,
            0x0, 0x1, 0x0, o, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        ]
    }
}

impl From<i32> for Orientation {
    fn from(f: i32) -> Self {
        Self(Self::libraw_flip_to_orientation(f))
    }
}
