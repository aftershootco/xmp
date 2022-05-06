use std::panic::Location;

#[derive(Debug)]
pub struct XmpError {
    inner: Box<dyn std::error::Error>,
    location: Location<'static>,
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
    #[error("{0}")]
    BuilderError(#[from] crate::ResultsBuilderError),
    #[cfg(feature = "jpeg")]
    #[error("Couldn't find xmp metadata in JFIF header")]
    JFIFHeaderMissing,
    #[cfg(feature = "jpeg")]
    #[error("{0}")]
    JfifError(#[from] jfifdump::JfifError),
    #[error("Invalid filetype")]
    InvalidFileType,
    #[error("{0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[cfg(feature = "jpeg")]
    #[error("{0}")]
    ImgParts(#[from] img_parts::Error),
    #[cfg(feature = "jpeg")]
    #[error("{0}")]
    ExifError(#[from] exif::Error),
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

impl std::error::Error for XmpError {}

impl<T: 'static> From<T> for XmpError
where
    T: Into<XmpErrorKind> + std::error::Error,
{
    #[track_caller]
    fn from(e: T) -> Self {
        Self {
            inner: Box::new(e),
            location: *Location::caller(),
        }
    }
}
