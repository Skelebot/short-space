use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error")]
    Io(#[source] io::Error),
    #[error("Failed to read CString from file that contains 0")]
    FileContainsNil,
    #[error("Failed get executable path")]
    FailedToGetExePath,
    #[error("Failed to load image")]
    FailedToLoadImage(#[source] image::ImageError),
    #[error("Image {name} is not RGBA")]
    ImageIsNotRgba { name: String },
    #[error("Failed to load obj file {name}")]
    FailedToLoadObj {
        name: String,
        #[source]
        inner: super::obj::Error,
    },
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        Error::Io(other)
    }
}

impl From<image::ImageError> for Error {
    fn from(other: image::ImageError) -> Self {
        Error::FailedToLoadImage(other)
    }
}

