use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PaperVersions {
    pub versions: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct PaperVersionBuilds {
    pub builds: Vec<u32>
}
pub type PaperVersion = String;

pub type PaperDownloadLink = String;