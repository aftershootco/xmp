use chrono::{DateTime, NaiveDateTime, Utc};
use minidom::Element;
use std::clone::Clone;
use std::fmt::Debug;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;
use std::marker::PhantomData;
use std::path::Path;

#[macro_use]
extern crate derive_builder;

#[cfg(feature = "jpeg")]
mod jpg;

pub mod errors;
use errors::{XmpError, XmpErrorKind};

mod traits;
use traits::*;

pub trait Image: Debug + Clone {}

#[derive(Debug, Clone, Copy, Default)]
pub struct Jpeg;
#[derive(Debug, Clone, Copy, Default)]
pub struct Raw;

impl Image for Jpeg {}
impl Image for Raw {}

#[derive(Debug, Builder, Default)]
pub struct Results<T: Image> {
    pub stars: u8,
    pub colors: String,
    pub datetime: i64,
    __marker: PhantomData<T>,
}

// impl ImageType {}

impl Results<Raw> {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, XmpError> {
        let str = std::fs::read_to_string(path)?;
        Self::from_slice(str.as_bytes())
    }
}

impl<T> Results<T>
where
    T: Image,
{
    pub fn from_slice<R>(bytes: R) -> Result<Self, XmpError>
    where
        R: BufRead,
    {
        let mut reader = quick_xml::Reader::from_reader(bytes);
        let xmpmeta: Element = Element::from_reader(&mut reader)?;
        let rdf = xmpmeta
            // .get_child("RDF", "http://www.w3.org/1999/02/22-rdf-syntax-ns#")
            .get_child("RDF", minidom::NSChoice::Any)
            .unrwrap_or_err(|| XmpErrorKind::ChildNotFound)?;
        let description = rdf
            // .get_child("Description", "http://www.w3.org/1999/02/22-rdf-syntax-ns#")
            .get_child("Description", minidom::NSChoice::Any)
            .unrwrap_or_err(|| XmpErrorKind::ChildNotFound)?;

        let mut results_builder = ResultsBuilder::default();
        results_builder.__marker(PhantomData);
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
        Ok(results_builder.build()?)
    }
}

#[derive(Debug, Default)]
pub struct UpdateResults<T: Image> {
    pub stars: Option<u8>,
    pub colors: Option<String>,
    pub datetime: Option<i64>,
    __marker: PhantomData<T>,
}

impl UpdateResults<Raw> {
    pub fn update(&self, path: impl AsRef<Path>) -> Result<(), XmpError> {
        let xml = self.update_xml(std::fs::read_to_string(&path)?)?;
        let mut f = std::fs::File::create(path)?;
        let mut bf = BufWriter::new(&mut f);
        bf.write_all(xml.as_bytes())?;

        Ok(())
    }
}

impl<T> UpdateResults<T>
where
    T: Image,
{
    pub fn update_xml(&self, xml: String) -> Result<String, XmpError> {
        let mut xmpmeta: Element = xml.parse()?;
        let rdf = xmpmeta
            // .get_child_mut("RDF", "http://www.w3.org/1999/02/22-rdf-syntax-ns#")
            .get_child_mut("RDF", minidom::NSChoice::Any)
            .unrwrap_or_err(|| XmpErrorKind::ChildNotFound)?;
        let description = rdf
            // .get_child_mut("Description", "http://www.w3.org/1999/02/22-rdf-syntax-ns#")
            .get_child_mut("Description", minidom::NSChoice::Any)
            .unrwrap_or_err(|| XmpErrorKind::ChildNotFound)?;

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

        let mut xml = Vec::new();
        // let mut bxml = BufWriter::new(&mut xml);
        xmpmeta.write_to(&mut xml)?;
        Ok(String::from_utf8(xml)?)
    }
}

#[test]
pub fn xmp_file() {
    println!("{:?}", Results::<Raw>::load("assets/file.xmp").unwrap());
}

#[test]
pub fn set_xmp() {
    let x = UpdateResults {
        colors: Some(String::from("Blue")),
        ..Default::default()
    };
    UpdateResults::<Raw>::update(&x, "assets/f.xmp").unwrap();
    println!("{:?}", Results::<Raw>::load("assets/f.xmp").unwrap());
}
