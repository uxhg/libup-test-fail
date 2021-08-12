use std::fmt;
use std::str::Utf8Error;
use url::ParseError;

#[derive(Debug)]
pub enum ErrorKind {
    PathNotExist(String),
    CSVErr(csv::Error),
    IOErr(std::io::Error),
    ObjConstructErr(String),
    CallToExtCommandErr(String),
    ExtCommandFailure(String),
    Git2Err(git2::Error),
    UrlParseErr(url::ParseError),
    Others(String),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ErrorKind::*;

        match *self {
            PathNotExist(ref path) => write!(f, "Path {} does not exist", path),
            CSVErr(ref csv_err) => write!(f, "Error from handling CSV: {}", csv_err.to_string()),
            ObjConstructErr(ref msg) => {
                write!(f, "Error when constructing object: {}", msg)
            }
            IOErr(ref io_err) => {
                write!(f, "Error during IO: {}", io_err.to_string())
            }
            CallToExtCommandErr(ref cmd) => write!(f, "Call to external command {} failed", cmd),
            ExtCommandFailure(ref cmd) => write!(f, "External command {} exited with failure", cmd),
            Git2Err(ref git2_err) => {
                write!(f, "Error by git2: {}", git2_err.to_string())
            }
            UrlParseErr(ref parse_err) => {
                write!(f, "Error by url: {}", parse_err.to_string())
            }
            Others(ref msg) => {
                write!(f, "Error: {}", msg)
            }
        }
    }
}


#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}" , self.kind())
    }
}

impl Error{
    pub(crate) fn new(kind: ErrorKind) -> Error {
        Error{kind}
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl From<csv::Error> for Error {
    fn from(e: csv::Error) -> Self {
        Error::new(ErrorKind::CSVErr(e))
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::new(ErrorKind::IOErr(e))
    }
}

impl From<git2::Error> for Error {
    fn from(e: git2::Error) -> Self {
        Error::new(ErrorKind::Git2Err(e))
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(e: Utf8Error) -> Self {
        Error::new(ErrorKind::Others(e.to_string()))
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::new(ErrorKind::Others(e.to_string()))
    }
}
impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Error::new(ErrorKind::UrlParseErr(e))
    }
}
