use chrono::{DateTime, NaiveDateTime, Utc};
use minidom::Element;
use std::clone::Clone;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

// General namespaces used in xmp
// xmlns:xmp="http://ns.adobe.com/xap/1.0/"
// xmlns:crd="http://ns.adobe.com/camera-raw-defaults/1.0/"
// xmlns:photoshop="http://ns.adobe.com/photoshop/1.0/"
// xmlns:stCamera="http://ns.adobe.com/photoshop/1.0/camera-profile"
// xmlns:crlcp="http://ns.adobe.com/camera-raw-embedded-lens-profile/1.0/"
// xmlns:tiff="http://ns.adobe.com/tiff/1.0/"
// xmlns:exif="http://ns.adobe.com/exif/1.0/"
// xmlns:aux="http://ns.adobe.com/exif/1.0/aux/"
// xmlns:exifEX="http://cipa.jp/exif/1.0/"
// xmlns:xmpMM="http://ns.adobe.com/xap/1.0/mm/"
// xmlns:stEvt="http://ns.adobe.com/xap/1.0/sType/ResourceEvent#"
// xmlns:dc="http://purl.org/dc/elements/1.1/"
// xmlns:xmpRights="http://ns.adobe.com/xap/1.0/rights/"
// xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"
// xmlns:Iptc4xmpCore="http://iptc.org/std/Iptc4xmpCore/1.0/xmlns/"
pub const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
pub const DC: &str = "http://purl.org/dc/elements/1.1/";
pub const TIFF: &str = "http://ns.adobe.com/tiff/1.0/";
pub const LR: &str = "http://ns.adobe.com/lightroom/1.0/";
pub const XMP: &str = "http://ns.adobe.com/xap/1.0/";
pub const EXIF: &str = "http://ns.adobe.com/exif/1.0/";
pub const PHOTOSHOP: &str = "http://ns.adobe.com/photoshop/1.0/";

/// make_item!(EXIF, "DateTimeOriginal")
/// expands to
/// pub const EXIF_DATETIMEORIGINAL: XmpItem = XmpItem {
///     name: "DateTimeOriginal",
///     namespace: "http://ns.adobe.com/exif/1.0/",
///     namespace_short: "exif",
/// };
#[macro_export]
macro_rules! make_item {
    ($ns:expr, $item:expr) => {
        paste::paste! {
            pub const [<$ns:upper _ $item:upper>]: XmpItem = XmpItem {
                name: $item,
                namespace: $ns,
                namespace_short: stringify!([<$ns:lower>]),
            };
        }
    };
}

make_item!(EXIF, "DateTimeOriginal");
make_item!(TIFF, "Orientation");
make_item!(XMP, "CreateDate");
make_item!(XMP, "Rating");
make_item!(XMP, "Label");
make_item!(PHOTOSHOP, "SidecarForExtension");

const XMP_EXT: [&str; 1] = ["xmp"];
const RAW_EXT: [&str; 37] = [
    "nef", "3fr", "ari", "arw", "bay", "crw", "cr2", "cr3", "cap", "dcs", "dcr", "dng", "drf",
    "eip", "erf", "fff", "gpr", "mdc", "mef", "mos", "mrw", "nrw", "obm", "orf", "pef", "ptx",
    "pxn", "r3d", "raw", "rwl", "rw2", "rwz", "sr2", "srf", "srw", "x3f", "raf",
];
const JPG_EXT: [&str; 4] = ["jpg", "jpeg", "avif", "hif"];
const PNG_EXT: [&str; 1] = ["png"];
const HEIF_EXT: [&str; 2] = ["heic", "heif"];
const TIFF_EXT: [&str; 2] = ["tiff", "tif"];

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct XmpItem<'xmp> {
    pub name: &'xmp str,
    pub namespace: &'xmp str,
    pub namespace_short: &'xmp str,
}

// pub type XmpItems = IntoIterator<Target = XmpItem>;

impl XmpItem<'_> {
    pub fn attr_name(&self) -> String {
        [self.namespace_short, self.name].join(":")
    }
    pub fn child_name(&self) -> &str {
        self.name
    }
}

#[macro_use]
extern crate derive_builder;

#[cfg(feature = "jpeg")]
mod jpg;

#[cfg(feature = "raw")]
mod raw;

#[cfg(feature = "png")]
mod png;

pub mod orientation;

pub mod time;
mod xml;

const DEFAULT_XML: &str = include_str!("default.xmp");

pub mod errors;
use errors::{XmpError, XmpErrorKind};

mod traits;
use traits::*;

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum ImageType {
    Raw,
    Xmp,
    Jpg,
    Png,
    Tiff,
    Heif,
    Others,
}

impl ImageType {
    pub fn from_path(p: impl AsRef<Path>) -> Self {
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
                } else if PNG_EXT.contains(&ext.as_str()) {
                    Self::Png
                } else if TIFF_EXT.contains(&ext.as_str()) {
                    Self::Tiff
                } else if HEIF_EXT.contains(&ext.as_str()) {
                    Self::Heif
                } else {
                    Self::Others
                }
            })
            .unwrap_or(Self::Others)
    }

    // #[cfg(feature = "magic")]
    // pub fn from_magic(p: impl AsRef<Path>) -> Self {
    //     use magic::{Cookie, CookieFlags};
    //     let cookie = magic::Cookie::open(CookieFlags::default()).unwrap();
    // }
}

#[derive(Debug, Clone, Default)]
pub struct UpdateOptions {
    indent: Option<(u8, usize)>,
    overwrite: bool,
}

#[derive(Debug, Default, Builder, PartialEq, Eq)]
pub struct UpdateResults {
    pub stars: Option<u8>,
    pub colors: Option<String>,
    /// Should always be utc
    pub datetime: Option<i64>,
    pub subjects: Option<Vec<String>>,
    pub hierarchies: Option<Vec<String>>,
    pub orientation: Option<usize>,
    /// Should be +-seconds based on whether the offset was specified
    pub offset: Option<i64>,
    pub sidecar_for_extension: Option<String>,
}

impl UpdateResults {
    pub fn update_xml<R>(&self, reader: R, options: UpdateOptions) -> Result<Vec<u8>, XmpError>
    where
        R: BufRead + Seek,
    {
        let mut xmpmeta = try_load_element(reader)?;

        let description = xmpmeta
            .get_child_mut("RDF", RDF)
            .and_then(|rdf| rdf.get_child_mut("Description", RDF))
            .otor(|| XmpErrorKind::ChildNotFound)?;

        description.add_prefixes([("xmp", XMP), ("exif", EXIF), ("tiff", TIFF)])?;

        if let Some(stars) = self.stars {
            description.set_attr("xmp:Rating", stars)
        }

        if let Some(colors) = &self.colors {
            description.set_attr("xmp:Label", colors)
        }

        if let Some(orientation) = &self.orientation {
            description.set_attr("tiff:Orientation", orientation.to_string())
        }

        if let Some(datetime) = self.datetime {
            let offset = if let Some(offset) = self.offset {
                chrono::FixedOffset::east_opt(offset as i32)
                    .unwrap_or_else(|| chrono::FixedOffset::east(0))
            } else {
                chrono::FixedOffset::east(0)
            };

            let datetime =
                DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(datetime, 0), Utc)
                    .with_timezone(&offset);

            if description.has_child("DateTimeOriginal", EXIF) {
                description.remove_child("DateTimeOriginal", EXIF);
            }
            description.set_attr("exif:DateTimeOriginal", &datetime.to_rfc3339());
        }

        if let Some(ref subjects) = self.subjects {
            let subjects: HashSet<String> = subjects.iter().cloned().collect();
            if let Some(bag) = description
                .get_child_mut("subject", DC)
                .and_then(|subjects| subjects.get_child_mut("Bag", RDF))
            {
                let existing: HashSet<String> = bag
                    .children()
                    .filter(|ch| ch.is("li", RDF))
                    .map(|li| li.text())
                    .collect();
                if existing != subjects {
                    let mut all = HashSet::new();
                    if !options.overwrite {
                        all.extend(existing);
                    }
                    all.extend(subjects);
                    println!("{:?}", all);
                    let list: Vec<Element> = all
                        .iter()
                        .map(|s| Element::builder("li", RDF).append(s.as_str()).build())
                        .collect();

                    *bag = Element::builder("Bag", RDF).append_all(list).build();
                }
            } else {
                let subjects: Vec<Element> = subjects
                    .iter()
                    .map(|s| Element::builder("li", RDF).append(s.as_str()).build())
                    .collect();
                let dc_subjects = Element::builder("subject", DC)
                    .append(Element::builder("Bag", RDF).append_all(subjects).build())
                    .build();
                description.remove_child("subject", DC);
                description.append_child(dc_subjects);
            }
        }
        if let Some(ref hierarchies) = self.hierarchies {
            let hierarchies: HashSet<String> = hierarchies.iter().cloned().collect();
            if let Some(bag) = description
                .get_child_mut("hierarchicalSubject", LR)
                .and_then(|hierarchy| hierarchy.get_child_mut("Bag", RDF))
            {
                let existing: HashSet<String> = bag
                    .children()
                    .filter(|ch| ch.is("li", RDF))
                    .map(|li| li.text())
                    .collect();
                // Only update if the new is not the same as the existing
                if existing != hierarchies {
                    let mut all = HashSet::new();
                    if !options.overwrite {
                        all.extend(existing);
                    }
                    all.extend(hierarchies);
                    let list: Vec<Element> = all
                        .iter()
                        .map(|s| Element::builder("li", RDF).append(s.as_str()).build())
                        .collect();
                    *bag = Element::builder("Bag", RDF)
                        .append_all(bag.children().cloned().collect::<Vec<Element>>())
                        .append_all(list)
                        .build()
                }
            } else {
                let hierarchies: Vec<Element> = hierarchies
                    .iter()
                    .map(|s| Element::builder("li", RDF).append(s.as_str()).build())
                    .collect();
                let lr_hierarchicalsubjects = Element::builder("hierarchicalSubject", LR)
                    .append(Element::builder("Bag", RDF).append_all(hierarchies).build())
                    .build();
                description.remove_child("hierarchicalSubject", LR);
                description.append_child(lr_hierarchicalsubjects);
            }
        }

        let mut xml = Vec::new();
        if let Some(indent) = options.indent {
            let mut qwriter = quick_xml::Writer::new_with_indent(&mut xml, indent.0, indent.1);
            xmpmeta.to_writer(&mut qwriter)?;
        } else {
            xmpmeta.write_to(&mut xml)?;
        }
        Ok(xml)
    }

    pub fn from_reader<R>(reader: R) -> Result<Self, XmpError>
    where
        R: BufRead + Seek,
    {
        // let mut reader = quick_xml::Reader::from_reader(bytes);
        // let xmpmeta: Element = Element::from_reader(&mut reader)?;
        let xmpmeta: Element = try_load_element(reader)?;
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
        results_builder.orientation(None);
        results_builder.offset(None);
        results_builder.sidecar_for_extension(None);

        if let Ok(v) = try_get_item(description, EXIF_DATETIMEORIGINAL) {
            // results_builder.datetime(crate::time::timestamp(&v.text()));
            let t = crate::time::timestamp_offset(&v);

            results_builder.datetime(t.map(|d| d.0));
            results_builder.offset(t.and_then(|d| d.1));
        }

        if let Ok(v) = try_get_item(description, PHOTOSHOP_SIDECARFOREXTENSION) {
            results_builder.sidecar_for_extension(Some(v));
        }

        if let Ok(v) = try_get_item(description, XMP_CREATEDATE) {
            let datetime = crate::time::timestamp_offset(&v);
            if datetime.is_some()
                && results_builder
                    .datetime
                    .map(|d| d.is_none())
                    .unwrap_or(false)
            {
                results_builder.datetime(datetime.map(|d| d.0));
                results_builder.offset(datetime.and_then(|d| d.1));
            }
        }
        if let Ok(v) = try_get_item(description, XMP_RATING) {
            results_builder.stars(v.parse().ok());
        }
        if let Ok(v) = try_get_item(description, XMP_LABEL) {
            results_builder.colors(Some(v));
        }

        if let Ok(v) = try_get_item(description, TIFF_ORIENTATION) {
            results_builder.orientation(v.parse().ok());
        }
        let subjects = description
            .get_child("subject", DC)
            .and_then(|subject| subject.get_child("Bag", RDF))
            .map(|bag| bag.children().map(|li| li.text()).collect::<Vec<String>>());

        results_builder.subjects(subjects);

        if let Some(o) = description
            .get_child("Orientation", TIFF)
            .and_then(|o| o.text().parse().ok())
        {
            results_builder.orientation(Some(o));
        };

        let hierarchies = description
            .get_child("hierarchicalSubject", LR)
            .and_then(|hierarchies| hierarchies.get_child("Bag", RDF))
            .map(|bag| bag.children().map(|li| li.text()).collect::<Vec<String>>());
        results_builder.hierarchies(hierarchies);

        Ok(results_builder.build()?)
    }

    #[inline]
    pub fn update(&self, path: impl AsRef<Path>) -> Result<(), XmpError> {
        self.write_to_with_options(path, Default::default())
    }

    #[inline]
    pub fn write_to_with_options(
        &self,
        path: impl AsRef<Path>,
        options: UpdateOptions,
    ) -> Result<(), XmpError> {
        let img_type = ImageType::from_path(&path);
        match img_type {
            ImageType::Jpg => self.update_jpg(path, options),
            ImageType::Png => self.update_png(path, options),
            ImageType::Raw => {
                if let Some(path) = exists_with_extension(&path, "xmp") {
                    self.update_xmp(path, options)
                } else {
                    eprintln!("\x1b[31mRaw files not supported and xmp file not found\x1b[0m");
                    Err(XmpError::from(XmpErrorKind::InvalidFileType))
                }
            }
            ImageType::Xmp => self.update_xmp(path, options),
            _ => Err(XmpError::from(XmpErrorKind::InvalidFileType)),
        }
    }
    #[inline]
    pub fn write_to(&self, path: impl AsRef<Path>) -> Result<(), XmpError> {
        self.write_to_with_options(path, Default::default())
    }
}

pub type OptionalResults = UpdateResults;

impl OptionalResults {
    #[inline]
    pub fn load(path: impl AsRef<Path>) -> Result<Self, XmpError> {
        let img_type = ImageType::from_path(&path);
        match img_type {
            ImageType::Xmp => OptionalResults::load_xmp(path),
            ImageType::Jpg => OptionalResults::load_jpg(path),
            ImageType::Png => OptionalResults::load_png(path),
            ImageType::Raw => {
                let raw_ext = path.as_ref().extension().and_then(OsStr::to_str);
                if let Some(path) = exists_with_extension(&path, "xmp") {
                    let xmp = OptionalResults::load_xmp(&path);
                    if let Ok(UpdateResults {
                        sidecar_for_extension: Some(ref sidecar_for_extension),
                        ..
                    }) = xmp
                    {
                        if let Some(ext) = raw_ext {
                            if sidecar_for_extension.eq_ignore_ascii_case(ext) {
                                return xmp;
                            }
                        }
                    } else {
                        return xmp;
                    }
                }
                OptionalResults::load_raw(path)
            }
            _ => Err(XmpError::from(XmpErrorKind::InvalidFileType)),
        }
    }
}

#[inline]
fn exists_with_extension(path: impl AsRef<Path>, ext: impl AsRef<OsStr>) -> Option<PathBuf> {
    path.as_ref().with_extension(ext).canonicalize().ok()
}

#[inline]
fn add_ns(ns: &str, buffer: impl BufRead) -> Result<Vec<u8>, XmpError> {
    use quick_xml::events::Event;

    let mut reader = quick_xml::Reader::from_reader(buffer);
    let mut writer = quick_xml::Writer::new(Vec::new());
    let mut buf = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) if e.name() == b"rdf:Description" => {
                let mut elem = e.clone();
                elem.clear_attributes();
                elem.extend_attributes(
                    e.attributes()
                        .map(|a| a.unwrap())
                        .filter(|a| a.key != b"xmlns:dc")
                        .into_iter(),
                );
                elem.push_attribute(("xmlns:dc", ns));
                writer.write_event(Event::Start(elem))?
            }
            Ok(Event::Eof) => break,
            Ok(elem) => writer.write_event(elem)?,
            Err(e) => return Err(e.into()),
        }
        buf.clear();
    }
    Ok(writer.into_inner())
}

pub fn try_load_element<R>(mut reader: R) -> Result<minidom::Element, XmpError>
where
    R: Read + Seek,
{
    let bfr = BufReader::new(&mut reader);
    let mut q_reader = quick_xml::Reader::from_reader(bfr);
    let xmpmeta: Element = match Element::from_reader(&mut q_reader) {
        Ok(xmp) => xmp,
        Err(e) => match e {
            minidom::Error::MissingNamespace => {
                // Since the bufreader has
                let mut bfr = BufReader::new(&mut reader);
                bfr.seek(SeekFrom::Start(0))?;
                let buffer = add_ns(DC, &mut bfr)?;
                let mut reader = quick_xml::Reader::from_reader(buffer.as_slice());
                Element::from_reader(&mut reader)?
            }
            _ => return Err(e.into()),
        },
    };
    Ok(xmpmeta)
}

pub fn try_get_item(element: &Element, item: XmpItem) -> Result<String, XmpError> {
    if let Some(s) = element.attr(&item.attr_name()) {
        Ok(s.to_owned())
    } else if let Some(e) = element.get_child(item.name, item.namespace) {
        Ok(e.text())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Failed to get Item {}", item.attr_name()),
        ))?
    }
}

pub fn try_get_description(element: &Element) -> Result<&Element, XmpError> {
    Ok(element
        .get_child("RDF", RDF)
        .and_then(|rdf| rdf.get_child("Description", RDF))
        .otor(|| XmpErrorKind::ChildNotFound)?)
}
