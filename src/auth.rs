use reqwest::{header::HeaderMap, Client};
use serde::{Deserialize, Serialize};

use crate::{error::Result, PixivApi};

const CLIENT_ID: &str = "MOBrBDS8blbauoSck0ZfDbtuzpyT";
const CLIENT_SECRET: &str = "lsACyCD94FhDUtGTXi3QzcFE2uU1hqtDaKeqrdwj";

const AUTH_TOKEN_URL: &str = "https://oauth.secure.pixiv.net/auth/token";

#[derive(Debug, Clone, Serialize)]
struct AuthTokenRequest {
    client_id: String,
    client_secret: String,
    grant_type: String,
    include_policy: bool,
    refresh_token: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AuthTokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
}

impl AuthTokenRequest {
    fn new(refresh_token: String) -> Self {
        Self {
            client_id: CLIENT_ID.to_string(),
            client_secret: CLIENT_SECRET.to_string(),
            grant_type: "refresh_token".to_string(),
            include_policy: true,
            refresh_token,
        }
    }
}

impl PixivApi {
    pub(crate) fn default_headers(language: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("app-os", "ios".parse().unwrap());
        headers.insert("app-os-version", "14.6".parse().unwrap());
        headers.insert(
            "user-agent",
            "PixivIOSApp/7.13.3 (iOS 14.6; iPhone13,2)".parse().unwrap(),
        );
        headers.insert("accept-language", language.parse().unwrap());

        headers
    }

    pub(crate) async fn refresh_token(&self) -> Result<u64> {
        let mut inner = self.inner.write().await;
        let resp: AuthTokenResponse = inner
            .client
            .post(AUTH_TOKEN_URL)
            .form(&AuthTokenRequest::new(inner.refresh_token.clone()))
            .send()
            .await?
            .json()
            .await?;

        let mut headers = Self::default_headers(&inner.language);
        headers.insert(
            "Authorization",
            format!("Bearer {}", resp.access_token).parse().unwrap(),
        );
        inner
            .persistent_token
            .save(resp.refresh_token.clone())
            .await;
        inner.refresh_token = resp.refresh_token;
        inner.client = Client::builder().default_headers(headers).build().unwrap();

        Ok(resp.expires_in)
    }

    pub(crate) async fn keep_refresh_token(api: PixivApi, mut expire_in: u64) {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(expire_in - 10)).await;
            expire_in = api.refresh_token().await.unwrap_or(10);
        }
    }
}
