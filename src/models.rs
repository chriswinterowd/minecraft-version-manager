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