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
    pub async fn new(refresh_token: impl PersistentToken + 'static, language: String) -> Self {
        let headers = Self::default_headers(&language);
        let client = Client::builder().default_headers(headers).build().unwrap();
        let api = PixivApi {
            inner: {
                Arc::new(RwLock::new(PixivApiInner {
                    refresh_token: refresh_token.load().await,
                    persistent_token: Box::new(refresh_token),
                    client,
                    language,
                }))
            },
        };
        // Get access token from refresh token on start.
        let expire_in = api.refresh_token().await.unwrap();
        // Keep refreshing token.
        tokio::spawn(Self::keep_refresh_token(api.clone(), expire_in));
        api
    }

    pub async fn new_no_refresh(access_token: String, language: String) -> Self {
        let mut headers = Self::default_headers(&language);
        headers.insert(
            "Authorization",
            format!("Bearer {}", access_token).parse().unwrap(),
        );
        let client = Client::builder().default_headers(headers).build().unwrap();
        PixivApi {
            inner: Arc::new(RwLock::new(PixivApiInner {
                refresh_token: "".to_string(),
                persistent_token: Box::new(NoPersistentToken),
                client,
                language,
            })),
        }
    }
}

#[async_trait]
pub trait PersistentToken: Send + Sync {
    async fn save(&self, refresh_token: String);
    async fn load(&self) -> String;
}

struct NoPersistentToken;

#[async_trait]
impl PersistentToken for NoPersistentToken {
    async fn save(&self, _: String) {}
    async fn load(&self) -> String {
        unreachable!()
    }
}

struct PixivApiInner {
    refresh_token: String,
    persistent_token: Box<dyn PersistentToken>,
    client: Client,
    language: String,
}
