use std::fmt;

#[derive(Debug)]
pub enum ErrorKind {
    PathNotExist(String),
    CSVErr(csv::Error),
    ObjConstructErr(String),
    CallToExtCommandErr(String),
    ExtCommandFailure(String),
    Others(String)
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
            CallToExtCommandErr(ref cmd) => write!(f, "Call to external command {} failed", cmd),
            ExtCommandFailure(ref cmd) => write!(f, "External command {} exited with failure", cmd),
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
