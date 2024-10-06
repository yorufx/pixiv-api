use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use reqwest::Client;
use tokio::sync::RwLock;

mod api;
mod auth;
mod error;
mod model;

pub use api::*;
pub use error::*;
pub use model::*;

#[derive(Clone)]
pub struct PixivApi {
    inner: Arc<RwLock<PixivApiInner>>,
}

impl Debug for PixivApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PixivApi").finish()
    }
}

impl PixivApi {
    pub async fn new(oauth_token: impl OAuthToken + 'static, language: Option<String>) -> Self {
        let headers = Self::default_headers(language.as_deref(), &oauth_token.access_token());
        let client = Client::builder().default_headers(headers).build().unwrap();
        let api = PixivApi {
            inner: {
                Arc::new(RwLock::new(PixivApiInner {
                    oauth_token: Box::new(oauth_token),
                    client,
                    language,
                }))
            },
        };
        // Keep refreshing token.
        tokio::spawn(Self::keep_refresh_token(api.clone()));
        api
    }

    pub async fn new_no_refresh(access_token: String, language: Option<String>) -> Self {
        let headers = Self::default_headers(language.as_deref(), &access_token);
        let client = Client::builder().default_headers(headers).build().unwrap();
        PixivApi {
            inner: Arc::new(RwLock::new(PixivApiInner {
                oauth_token: Box::new(NoRefreshToken { access_token }),
                client,
                language,
            })),
        }
    }
}

#[async_trait]
pub trait OAuthToken: Send + Sync {
    fn access_token(&self) -> String;
    fn refresh_token(&self) -> String;
    /// Unix timestamp in seconds.
    fn expires_at(&self) -> i64;
    async fn refresh(&mut self, access_token: String, refresh_token: String, expires_at: i64);
}

struct NoRefreshToken {
    access_token: String,
}

#[async_trait]
impl OAuthToken for NoRefreshToken {
    fn access_token(&self) -> String {
        self.access_token.clone()
    }

    fn refresh_token(&self) -> String {
        unreachable!()
    }

    fn expires_at(&self) -> i64 {
        0
    }

    async fn refresh(&mut self, _: String, _: String, _: i64) {
        unreachable!()
    }
}

struct PixivApiInner {
    oauth_token: Box<dyn OAuthToken>,
    client: Client,
    language: Option<String>,
}
