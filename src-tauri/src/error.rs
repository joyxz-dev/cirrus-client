use thiserror::Error;

#[derive(Error, Debug)]
pub enum CirrusError {
    #[error("Network: {0}")]
    Network(#[from] reqwest::Error),
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Auth: {0}")]
    Auth(String),
    #[error("Instance: {0}")]
    Instance(String),
    #[error("Launch: {0}")]
    Launch(String),
    #[error("Security: {0}")]
    Security(String),
    #[error("Store: {0}")]
    Store(String),
}

impl serde::Serialize for CirrusError {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(self.to_string().as_ref())
    }
}
