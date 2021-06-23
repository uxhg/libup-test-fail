use std::path::Path;
use log::error;
// use std::error::Error;
use crate::utils::err;
use csv::ReaderBuilder;
use std::convert::TryFrom;
use std::fmt;


pub static REL_KINDS: [&str; 3] = [
    "PomDepDataFlowLLib",
    "PomDepDataFlowRLib",
    "SameGroupDataFlowLib",
];


#[derive(Copy,Clone,Debug)]
pub enum RelKind {
    PomDepDataFlowLLib,
    PomDepDataFlowRLib,
    SameGroupDataFlowLib,
    Default,
}

impl fmt::Display for RelKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl RelKind {
    pub fn from_str(k: &str) -> Self {
        match k {
            "PomDepDataFlowLLib" => Self::PomDepDataFlowLLib,
            "PomDepDataFlowRLib" => Self::PomDepDataFlowRLib,
            "SameGroupDataFlowLib" => Self::SameGroupDataFlowLib,
            _ => Self::Default
        }
    }
}

pub trait LibPairRelation {
    fn to_id_pair(&self) -> (String, String);
}


pub struct SimpleLibPair {
    kind: RelKind,
    group_x: String,
    artifact_x: String,
    group_y: String,
    artifact_y: String
}

impl SimpleLibPair {
    pub fn new(k: RelKind, grp_x: &str, art_x: &str, grp_y: &str, art_y: &str) -> Self {
        SimpleLibPair {
            kind: k,
            group_x: String::from(grp_x),
            artifact_x: String::from(art_x),
            group_y: String::from(grp_y),
            artifact_y: String::from(art_y)
        }
    }
    pub fn kind(&self) -> RelKind {
        self.kind
    }
    pub fn group_x(&self) -> &str {
        &self.group_x
    }
    pub fn artifact_x(&self) -> &str {
        &self.artifact_x
    }
    pub fn group_y(&self) -> &str {
        &self.group_y
    }
    pub fn artifact_y(&self) -> &str {
        &self.artifact_y
    }
    pub fn set_kind(&mut self, kind: RelKind) {
        self.kind = kind
    }
}


impl LibPairRelation for SimpleLibPair {
    fn to_id_pair(&self) -> (String, String) {
        (format!("{}:{}:jar", self.group_x, self.artifact_x),
         format!("{}:{}:jar", self.group_y, self.artifact_y))
    }
}

impl fmt::Display for SimpleLibPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]\t{}:{} -> {}:{}", &self.kind, &self.group_x, &self.artifact_x,
               &self.group_y, &self.artifact_y)
    }
}
impl TryFrom<csv::StringRecord> for SimpleLibPair {
    type Error = err::Error;
    fn try_from(r: csv::StringRecord) -> Result<Self, Self::Error> {
        return if r.len() == 4 {
            let p: Vec<&str> = (0..4).map(|x| r.get(x).unwrap()).collect();
            Ok(SimpleLibPair::new(RelKind::Default, p[0], p[1], p[2], p[3]))
        } else if r.len() == 3 {
            let p: Vec<&str> = (0..3).map(|x| r.get(x).unwrap()).collect();
            Ok(SimpleLibPair::new(RelKind::Default, p[0], p[1], p[0], p[2]))
        } else {
            let err_msg = format!("Cannot convert {} to a SimpleLibPair", r.as_slice());
            Err(err::Error::new(err::ErrorKind::ObjConstructErr(err_msg)))
        }
    }
}


pub fn read_simple_lib_pair(in_f: &Path, k: RelKind) -> Result<Vec<SimpleLibPair>, err::Error> {
    let mut reader = ReaderBuilder::new().delimiter(b'\t').has_headers(false).from_path(in_f)?;
    Ok(reader.records().map(|x| -> Result<SimpleLibPair, err::Error> {
        let mut y = SimpleLibPair::try_from(x?)?;
        y.set_kind(k); Ok(y)}).collect::<Result<Vec<SimpleLibPair>, err::Error>>()?)
    /* for result in reader.records() {
        let row = result?;
        println!("{:?}", row);
    }*/
}


#[cfg(test)]
mod test{
    use crate::dl_relation;
    use std::path::Path;

    #[test]
    pub fn test_read_csv() {
        let csv_in = Path::new("data/for-tests/dlout/PomDepDataFlowLLib.csv");
        assert!(dl_relation::read_pom_dep_dataflow(csv_in).is_ok());
    }
}
