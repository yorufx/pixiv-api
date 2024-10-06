use crate::PixivError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Net(#[from] reqwest::Error),
    #[error(transparent)]
    Pixiv(#[from] PixivError),
    #[error("invalid url")]
    InvalidUrl(Option<String>),
}

pub type Result<T> = std::result::Result<T, Error>;
