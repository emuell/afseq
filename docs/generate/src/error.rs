use std::io;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("failed to execute lua-language-server")]
    Exec,
    #[error("unable to parse doc JSON")]
    JsonParse(#[from] serde_json::Error),
}
