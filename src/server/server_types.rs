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