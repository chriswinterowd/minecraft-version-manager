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

    pub fn get_server_path(&self) -> String {
        match self {
            ServerType::Vanilla => String::from("vanilla"),
            ServerType::Paper => String::from("paper")
        }
    }
}