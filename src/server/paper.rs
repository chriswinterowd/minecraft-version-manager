use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PaperVersions {
    versions: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct PaperVersionBuilds {
    builds: Vec<u32>
}
pub type PaperVersion = String;

pub type PaperDownloadLink = String;