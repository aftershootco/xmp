use chrono::{DateTime, NaiveDateTime, Utc};
use minidom::Element;
use minidom::NSChoice;
use std::clone::Clone;
use std::collections::HashSet;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;
use std::marker::PhantomData;
use std::path::Path;

const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const DC: &str = "http://purl.org/dc/elements/1.1/";
const LR: &str = "http://ns.adobe.com/lightroom/1.0/";

#[macro_use]
extern crate derive_builder;

#[cfg(feature = "jpeg")]
mod jpg;
#[cfg(feature = "jpeg")]
pub use jpg::*;

#[cfg(feature = "raw")]
mod raw;
#[cfg(feature = "raw")]
pub use raw::*;

const DEFAULT_XML: &str = include_str!("default.xmp");

pub mod errors;
use errors::{XmpError, XmpErrorKind};

mod traits;
use traits::*;

#[derive(Debug, Builder, Default)]
pub struct Results<T: Image> {
    pub stars: u8,
    pub colors: String,
    pub datetime: i64,
    pub subjects: Vec<String>,
    pub hierarchies: Vec<String>,
    __marker: PhantomData<T>,
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
            .get_child("RDF", NSChoice::Any)
            .otor(|| XmpErrorKind::ChildNotFound)?;
        let description = rdf
            // .get_child("Description", "http://www.w3.org/1999/02/22-rdf-syntax-ns#")
            .get_child("Description", NSChoice::Any)
            .otor(|| XmpErrorKind::ChildNotFound)?;

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
        let subjects = description
            .get_child("subject", NSChoice::Any)
            .and_then(|subject| subject.get_child("Bag", NSChoice::Any))
            .map(|bag| bag.children().map(|li| li.text()).collect::<Vec<String>>())
            .otor(|| XmpErrorKind::ChildNotFound)?;
        results_builder.subjects(subjects);

        let hierarchies = description
            .get_child("hierarchicalSubject", NSChoice::Any)
            .and_then(|hierarchies| hierarchies.get_child("Bag", NSChoice::Any))
            .map(|bag| bag.children().map(|li| li.text()).collect::<Vec<String>>())
            .otor(|| XmpErrorKind::ChildNotFound)?;
        results_builder.hierarchies(hierarchies);

        Ok(results_builder.build()?)
    }
}

#[derive(Debug, Default, Builder)]
// #[builder(pattern = "owned")
pub struct UpdateResults<T: Image> {
    pub stars: Option<u8>,
    pub colors: Option<String>,
    pub datetime: Option<i64>,
    pub subjects: Option<Vec<String>>,
    pub hierarchies: Option<Vec<String>>,
    __marker: PhantomData<T>,
}

impl<T> UpdateResults<T>
where
    T: Image,
{
    pub fn update_xml<R>(&self, bytes: R, indent: bool) -> Result<Vec<u8>, XmpError>
    where
        R: BufRead,
    {
        let mut reader = quick_xml::Reader::from_reader(bytes);
        let mut xmpmeta: Element = Element::from_reader(&mut reader)?;
        let rdf = xmpmeta
            // .get_child_mut("RDF", "http://www.w3.org/1999/02/22-rdf-syntax-ns#")
            .get_child_mut("RDF", NSChoice::Any)
            .otor(|| XmpErrorKind::ChildNotFound)?;
        let description = rdf
            // .get_child_mut("Description", "http://www.w3.org/1999/02/22-rdf-syntax-ns#")
            .get_child_mut("Description", NSChoice::Any)
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
            let lr_hierarchichalsubjects = Element::builder("hierarchicalSubject", LR)
                .append(Element::builder("Bag", RDF).append_all(hierarchies).build())
                .build();
            description.append_child(lr_hierarchichalsubjects);
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
    pub fn from_slice<R>(bytes: R) -> Result<Self, XmpError>
    where
        R: BufRead,
    {
        let mut reader = quick_xml::Reader::from_reader(bytes);
        let xmpmeta: Element = Element::from_reader(&mut reader)?;
        let rdf = xmpmeta
            // .get_child("RDF", "http://www.w3.org/1999/02/22-rdf-syntax-ns#")
            .get_child("RDF", NSChoice::Any)
            .otor(|| XmpErrorKind::ChildNotFound)?;
        let description = rdf
            // .get_child("Description", "http://www.w3.org/1999/02/22-rdf-syntax-ns#")
            .get_child("Description", NSChoice::Any)
            .otor(|| XmpErrorKind::ChildNotFound)?;

        let mut results_builder = UpdateResultsBuilder::default();
        results_builder.colors(None);
        results_builder.stars(None);
        results_builder.datetime(None);
        results_builder.subjects(None);
        results_builder.hierarchies(None);
        results_builder.__marker(PhantomData);
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
            .get_child("subject", NSChoice::Any)
            .and_then(|subject| subject.get_child("Bag", NSChoice::Any))
            .map(|bag| bag.children().map(|li| li.text()).collect::<Vec<String>>());

        results_builder.subjects(subjects);

        let hierarchies = description
            .get_child("hierarchicalSubject", NSChoice::Any)
            .and_then(|hierarchies| hierarchies.get_child("Bag", NSChoice::Any))
            .map(|bag| bag.children().map(|li| li.text()).collect::<Vec<String>>());
        results_builder.hierarchies(hierarchies);

        Ok(results_builder.build()?)
    }
}

pub type OptionalResults<T> = UpdateResults<T>;
