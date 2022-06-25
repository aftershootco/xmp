use std::ffi::{OsStr, OsString};
use std::panic::Location;

pub struct XmpError {
    inner: XmpErrorKind,
    name: Option<OsString>,
    location: Location<'static>,
}

impl XmpError {
    pub fn with_name(mut self, name: impl AsRef<OsStr>) -> Self {
        self.name = Some(name.as_ref().to_owned());
        self
    }
}

#[derive(Debug, thiserror::Error)]
pub enum XmpErrorKind {
    #[error("Child element not found")]
    ChildNotFound,
    #[error("XMP header / File missing")]
    XMPMissing,
    #[error("{0}")]
    MinidomError(#[from] minidom::Error),
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    // #[error("{0}")]
    // RequestBuilderError(#[from] crate::ResultsBuilderError),
    #[error("{0}")]
    OptionlRequestBuilderError(#[from] crate::UpdateResultsBuilderError),
    #[error("{0}")]
    QuickXml(#[from] quick_xml::Error),
    #[error("Invalid filetype")]
    InvalidFileType,
    #[error("{0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[cfg(feature = "jpeg")]
    #[error("Couldn't find xmp metadata in JFIF header")]
    JFIFHeaderMissing,
    #[cfg(feature = "jpeg")]
    #[error("{0}")]
    JfifError(#[from] jfifdump::JfifError),
    #[cfg(feature = "jpeg")]
    #[error("{0}")]
    ImgParts(#[from] img_parts::Error),
    #[cfg(feature = "jpeg")]
    #[error("{0}")]
    ExifError(#[from] exif::Error),

    #[cfg(feature = "raw")]
    #[error("{0}")]
    LibrawError(#[from] libraw_r::LibrawError),
}

impl std::fmt::Display for XmpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} at {}#{}:{}",
            self.inner,
            self.location.file(),
            self.location.line(),
            self.location.column()
        )
    }
}

impl std::fmt::Debug for XmpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            write!(
                f,
                "{:?} at {}#{}:{} for file {:?}",
                self.inner,
                self.location.file(),
                self.location.line(),
                self.location.column(),
                name
            )
        } else {
            write!(
                f,
                "{} at {}#{}:{}",
                self.inner,
                self.location.file(),
                self.location.line(),
                self.location.column()
            )
        }
    }
}

impl std::error::Error for XmpError {}

impl XmpError {
    pub fn kind(&self) -> &XmpErrorKind {
        &self.inner
    }
}
impl<T: 'static> From<T> for XmpError
where
    T: Into<XmpErrorKind> + std::error::Error,
{
    #[track_caller]
    fn from(e: T) -> Self {
        Self {
            inner: e.into(),
            name: None,
            location: *Location::caller(),
        }
    }
}
