use std::io::Write;

use log::{error, info, warn};
use reqwest::Url;

use crate::pomdep::MvnCoord;
use crate::utils::err;

pub fn get_jar_name(coord: &MvnCoord) -> String {
    format!("{}-{}.jar", coord.artifact_id(), coord.version_id())
}

pub async fn get_jar_remote<W: Write>(coord: &MvnCoord, out: &mut W) -> Result<usize, err::Error> {
    // let url_template = "https://repo1.maven.org/maven2/org/reflections/reflections/0.9.12/reflections-0.9.12.jar"
    let url_prefix = Url::parse("https://repo1.maven.org/maven2/").unwrap();
    let path_segments = format!("{}/{}/{}/{}",
                                &coord.group_id().replace(".", "/"),
                                &coord.artifact_id(),
                                &coord.version_id(),
                                &get_jar_name(&coord));
    let url = url_prefix.join(&path_segments)?;
    info!("Fetching file from {}", url);
    let contents = reqwest::get(url).await?.bytes().await?;
    info!("Get file of size: {}", contents.len());
    match out.write(contents.as_ref()) {
        Ok(u) => Ok(u),
        Err(e) => Err(err::Error::new(err::ErrorKind::IOErr(e)))
    }
}
