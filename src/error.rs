use crate::PixivError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Net(#[from] reqwest::Error),
    #[error(transparent)]
    Pixiv(#[from] PixivError),
}

pub type Result<T> = std::result::Result<T, Error>;
