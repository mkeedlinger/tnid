#[derive(Debug)]
pub enum Error {
    TimeError(Box<dyn std::error::Error>),
    InvalidUuidFormat,
    InvalidUuidVersion,
    InvalidNameEncoding,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::TimeError(err) => write!(f, "Time Error: {:?}", err),
            Error::InvalidUuidFormat => write!(f, "Invalid UUID format"),
            Error::InvalidUuidVersion => write!(f, "Invalid TNID: not UUIDv8 format"),
            Error::InvalidNameEncoding => write!(f, "Invalid TNID: invalid name encoding"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::TimeError(err) => Some(err.as_ref()),
            _ => None,
        }
    }
}