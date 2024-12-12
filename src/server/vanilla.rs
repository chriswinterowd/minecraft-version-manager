use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Latest {
    pub release: String
}

#[derive(Deserialize, Debug)]
pub struct VanillaVersions {
    pub latest: Latest,
    pub versions: Vec<VanillaVersionInfo>
}

#[derive(Deserialize, Debug)]
pub struct VanillaVersionInfo {
    pub id: String,
    pub url: String
}

pub type VanillaDownloadLink = String;

#[derive(Deserialize, Debug)]
pub struct VanillaDownloadInfo {
    pub url: VanillaDownloadLink
}
#[derive(Deserialize, Debug)]
pub struct ServerDownload {
    pub server: VanillaDownloadInfo
}
#[derive(Deserialize, Debug)]
pub struct VersionDownloads {
    pub downloads: ServerDownload
}