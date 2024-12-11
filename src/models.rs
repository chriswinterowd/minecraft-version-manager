use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Latest {
    pub release: String
}

#[derive(Deserialize, Debug)]
pub struct Versions {
    pub latest: Latest,
    pub versions: Vec<VersionInfo>
}

#[derive(Deserialize, Debug)]
pub struct VersionInfo {
    pub id: String,
    pub url: String
}

#[derive(Deserialize, Debug)]
pub struct DownloadLink {
    pub url: String
}
#[derive(Deserialize, Debug)]
pub struct ServerDownload {
    pub server: DownloadLink
}
#[derive(Deserialize, Debug)]
pub struct VersionDownloads {
    pub downloads: ServerDownload
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerType {
    Vanilla,
    Paper
}

impl ServerType {
    pub fn determine_server_type(paper: bool) -> Self {
        if paper {
            ServerType::Paper
        } else {
            ServerType::Vanilla
        }
    }
}