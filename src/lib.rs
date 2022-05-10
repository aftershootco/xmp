use chrono::{DateTime, NaiveDateTime, Utc};
use minidom::Element;
use std::clone::Clone;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const DC: &str = "http://purl.org/dc/elements/1.1/";
const LR: &str = "http://ns.adobe.com/lightroom/1.0/";

const XMP_EXT: [&str; 1] = ["xmp"];
const RAW_EXT: [&str; 37] = [
    "nef", "3fr", "ari", "arw", "bay", "crw", "cr2", "cr3", "cap", "dcs", "dcr", "dng", "drf",
    "eip", "erf", "fff", "gpr", "mdc", "mef", "mos", "mrw", "nrw", "obm", "orf", "pef", "ptx",
    "pxn", "r3d", "raw", "rwl", "rw2", "rwz", "sr2", "srf", "srw", "x3f", "raf",
];
const JPG_EXT: [&str; 9] = [
    "jpg", "jpeg", "png", "heic", "avif", "heif", "tiff", "tif", "hif",
];

#[macro_use]
extern crate derive_builder;

#[cfg(feature = "jpeg")]
mod jpg;

#[cfg(feature = "raw")]
mod raw;

const DEFAULT_XML: &str = include_str!("default.xmp");

pub mod errors;
use errors::{XmpError, XmpErrorKind};

mod traits;
use traits::*;

pub enum ImageType {
    Raw,
    Xmp,
    Jpg,
    Others,
}

impl<T> From<T> for ImageType
where
    T: AsRef<Path>,
{
    fn from(p: T) -> Self {
        p.as_ref()
            .extension()
            .and_then(OsStr::to_str)
            .map(str::to_ascii_lowercase)
            .map(|ext| {
                if JPG_EXT.contains(&ext.as_str()) {
                    Self::Jpg
                } else if RAW_EXT.contains(&ext.as_str()) {
                    Self::Raw
                } else if XMP_EXT.contains(&ext.as_str()) {
                    Self::Xmp
                } else {
                    Self::Others
                }
            })
            .unwrap_or(Self::Others)
    }
}

#[derive(Debug, Builder, Default)]
pub struct Results {
    pub stars: u8,
    pub colors: String,
    pub datetime: i64,
    pub subjects: Vec<String>,
    pub hierarchies: Vec<String>,
}

impl Results {
    pub fn from_reader<R>(reader: R) -> Result<Self, XmpError>
    where
        R: BufRead,
    {
        let mut reader = quick_xml::Reader::from_reader(reader);
        let xmpmeta: Element = Element::from_reader(&mut reader)?;
        let description = xmpmeta
            .get_child("RDF", RDF)
            .and_then(|rdf| rdf.get_child("Description", RDF))
            .otor(|| XmpErrorKind::ChildNotFound)?;

        let mut results_builder = ResultsBuilder::default();
        description.attrs().for_each(|attr| match attr {
            ("xmp:Label", v) => {
                results_builder.colors(v.to_owned());
            }
            ("xmp:Rating", v) => {
                results_builder.stars(v.parse().unwrap_or(0));
            }
            ("xmp:CreateDate", v) => {
                let datetime = chrono::DateTime::parse_from_rfc3339(v)
                    .map(|d| d.timestamp())
                    .unwrap_or(0);
                results_builder.datetime(datetime);
            }
            ("exif:DateTimeOriginal", v) => {
                let datetime = chrono::DateTime::parse_from_rfc3339(v)
                    .map(|d| d.timestamp())
                    .unwrap_or(0);
                results_builder.datetime(datetime);
            }
            _ => (),
        });
        let subjects = description
            .get_child("subject", DC)
            .and_then(|subject| subject.get_child("Bag", RDF))
            .map(|bag| bag.children().map(|li| li.text()).collect::<Vec<String>>())
            .otor(|| XmpErrorKind::ChildNotFound)?;
        results_builder.subjects(subjects);

        let hierarchies = description
            .get_child("hierarchicalSubject", LR)
            .and_then(|hierarchies| hierarchies.get_child("Bag", RDF))
            .map(|bag| bag.children().map(|li| li.text()).collect::<Vec<String>>())
            .otor(|| XmpErrorKind::ChildNotFound)?;
        results_builder.hierarchies(hierarchies);

        Ok(results_builder.build()?)
    }

    #[inline]
    pub fn load(path: impl AsRef<Path>) -> Result<Self, XmpError> {
        let img_type = ImageType::from(&path);
        match img_type {
            ImageType::Jpg => Self::load_jpg(path),
            ImageType::Raw => {
                if let Some(path) = exists_with_extension(&path, "xmp") {
                    Self::load_raw(path)
                } else {
                    eprintln!("\x1b[31mRaw files not supported and xmp file not found\x1b[0m");
                    Err(XmpError::from(XmpErrorKind::InvalidFileType))
                }
            }
            ImageType::Xmp => Self::load_raw(path),
            ImageType::Others => Err(XmpError::from(XmpErrorKind::InvalidFileType)),
        }
    }
}

#[derive(Debug, Default, Builder)]
// #[builder(pattern = "owned")
pub struct UpdateResults {
    pub stars: Option<u8>,
    pub colors: Option<String>,
    pub datetime: Option<i64>,
    pub subjects: Option<Vec<String>>,
    pub hierarchies: Option<Vec<String>>,
}

impl UpdateResults {
    pub fn update_xml<R>(&self, bytes: R, indent: bool) -> Result<Vec<u8>, XmpError>
    where
        R: BufRead,
    {
        let mut reader = quick_xml::Reader::from_reader(bytes);
        let mut xmpmeta: Element = Element::from_reader(&mut reader)?;
        let description = xmpmeta
            .get_child_mut("RDF", RDF)
            .and_then(|rdf| rdf.get_child_mut("Description", RDF))
            .otor(|| XmpErrorKind::ChildNotFound)?;

        if let Some(stars) = self.stars {
            description.set_attr("xmp:Rating", stars)
        }
        if let Some(colors) = &self.colors {
            description.set_attr("xmp:Label", colors)
        }

        if let Some(datetime) = self.datetime {
            let datetime =
                DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(datetime, 0), Utc)
                    .to_rfc3339();
            description.set_attr("xmp:CreateDate", &datetime);
            description.set_attr("exif:DateTimeOriginal", datetime);
        }
        if let Some(ref subjects) = self.subjects {
            let mut subjects: HashSet<String> = subjects.iter().cloned().collect();
            if let Some(bag) = description
                .get_child_mut("subject", DC)
                .and_then(|subjects| subjects.get_child_mut("Bag", RDF))
            {
                subjects.extend(bag.children().map(|li| li.text()));
                description.remove_child("subject", DC);
            }
            let subjects: Vec<Element> = subjects
                .iter()
                .map(|s| Element::builder("li", RDF).append(s.as_str()).build())
                .collect();
            let dc_subjects = Element::builder("subject", DC)
                .append(Element::builder("Bag", RDF).append_all(subjects).build())
                .build();
            description.append_child(dc_subjects);
        }
        if let Some(ref hierarchies) = self.hierarchies {
            let mut hierarchies: HashSet<String> = hierarchies.iter().cloned().collect();
            if let Some(bag) = description
                .get_child_mut("hierarchicalSubject", LR)
                .and_then(|hierarchy| hierarchy.get_child_mut("Bag", RDF))
            {
                hierarchies.extend(bag.children().map(|li| li.text()));
                description.remove_child("hierarchicalSubject", LR);
            };
            let hierarchies: Vec<Element> = hierarchies
                .iter()
                .map(|s| Element::builder("li", RDF).append(s.as_str()).build())
                .collect();
            let lr_hierarchicalsubjects = Element::builder("hierarchicalSubject", LR)
                .append(Element::builder("Bag", RDF).append_all(hierarchies).build())
                .build();
            description.append_child(lr_hierarchicalsubjects);
        }

        let mut xml = Vec::new();
        if indent {
            let mut qwriter = quick_xml::Writer::new_with_indent(&mut xml, b' ', 4);
            xmpmeta.to_writer(&mut qwriter)?;
        } else {
            xmpmeta.write_to(&mut xml)?;
        }
        Ok(xml)
    }

    pub fn from_reader<R>(bytes: R) -> Result<Self, XmpError>
    where
        R: BufRead,
    {
        let mut reader = quick_xml::Reader::from_reader(bytes);
        let xmpmeta: Element = Element::from_reader(&mut reader)?;
        let description = xmpmeta
            .get_child("RDF", RDF)
            .and_then(|rdf| rdf.get_child("Description", RDF))
            .otor(|| XmpErrorKind::ChildNotFound)?;

        let mut results_builder = UpdateResultsBuilder::default();
        results_builder.colors(None);
        results_builder.stars(None);
        results_builder.datetime(None);
        results_builder.subjects(None);
        results_builder.hierarchies(None);
        description.attrs().for_each(|attr| match attr {
            ("xmp:Label", v) => {
                results_builder.colors(Some(v.to_owned()));
            }
            ("xmp:Rating", v) => {
                results_builder.stars(v.parse().ok());
            }
            ("xmp:CreateDate", v) => {
                let datetime = chrono::DateTime::parse_from_rfc3339(v).map(|d| d.timestamp());
                results_builder.datetime(datetime.ok());
            }
            ("exif:DateTimeOriginal", v) => {
                let datetime = chrono::DateTime::parse_from_rfc3339(v).map(|d| d.timestamp());
                results_builder.datetime(datetime.ok());
            }
            _ => (),
        });
        let subjects = description
            .get_child("subject", DC)
            .and_then(|subject| subject.get_child("Bag", RDF))
            .map(|bag| bag.children().map(|li| li.text()).collect::<Vec<String>>());

        results_builder.subjects(subjects);

        let hierarchies = description
            .get_child("hierarchicalSubject", LR)
            .and_then(|hierarchies| hierarchies.get_child("Bag", RDF))
            .map(|bag| bag.children().map(|li| li.text()).collect::<Vec<String>>());
        results_builder.hierarchies(hierarchies);

        Ok(results_builder.build()?)
    }

    #[inline]
    pub fn update(&self, path: impl AsRef<Path>) -> Result<(), XmpError> {
        self.write_to(path)
    }

    #[inline]
    pub fn write_to(&self, path: impl AsRef<Path>) -> Result<(), XmpError> {
        let img_type = ImageType::from(&path);
        match img_type {
            ImageType::Jpg => self.update_jpg(path),
            ImageType::Raw => {
                if let Some(path) = exists_with_extension(&path, "xmp") {
                    self.update_raw(path)
                } else {
                    eprintln!("\x1b[31mRaw files not supported and xmp file not found\x1b[0m");
                    Err(XmpError::from(XmpErrorKind::InvalidFileType))
                }
            }
            ImageType::Xmp => self.update_raw(path),
            ImageType::Others => Err(XmpError::from(XmpErrorKind::InvalidFileType)),
        }
    }
}

pub type OptionalResults = UpdateResults;

impl OptionalResults {
    #[inline]
    pub fn load(path: impl AsRef<Path>) -> Result<Self, XmpError> {
        let img_type = ImageType::from(&path);
        match img_type {
            ImageType::Jpg => OptionalResults::load_jpg(path),
            ImageType::Raw => {
                if let Some(path) = exists_with_extension(&path, "xmp") {
                    OptionalResults::load_raw(path)
                } else {
                    eprintln!("\x1b[31mRaw files not supported and xmp file not found\x1b[0m");
                    Err(XmpError::from(XmpErrorKind::InvalidFileType))
                }
            }
            ImageType::Xmp => OptionalResults::load_raw(path),
            ImageType::Others => Err(XmpError::from(XmpErrorKind::InvalidFileType)),
        }
    }
}

#[inline]
fn exists_with_extension(path: impl AsRef<Path>, ext: impl AsRef<OsStr>) -> Option<PathBuf> {
    let new_path = path.as_ref().with_extension(ext);
    // println!("{:?}", path);
    if new_path.exists() && new_path == path.as_ref().to_path_buf() {
        std::fs::canonicalize(path).ok()
    } else {
        None
    }
}
