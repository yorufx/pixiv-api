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
    expires_in: i64,
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
    pub(crate) fn default_headers(language: Option<&str>, access_token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("app-os", "ios".parse().unwrap());
        headers.insert("app-os-version", "14.6".parse().unwrap());
        headers.insert(
            "user-agent",
            "PixivIOSApp/7.13.3 (iOS 14.6; iPhone13,2)".parse().unwrap(),
        );
        headers.insert(
            "Authorization",
            format!("Bearer {}", access_token).parse().unwrap(),
        );
        if let Some(language) = language {
            headers.insert("accept-language", language.parse().unwrap());
        }

        headers
    }

    pub(crate) async fn refresh_token(&self) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        // Refresh access token.
        let mut inner = self.inner.write().await;
        let resp: AuthTokenResponse = inner
            .client
            .post(AUTH_TOKEN_URL)
            .form(&AuthTokenRequest::new(inner.oauth_token.refresh_token()))
            .send()
            .await?
            .json()
            .await?;

        // Update client headers.
        let headers = Self::default_headers(inner.language.as_deref(), &resp.access_token);
        inner.client = Client::builder().default_headers(headers).build().unwrap();
        println!("Refreshed token: {:#?}", resp);

        // Save new tokens.
        inner
            .oauth_token
            .refresh(
                resp.access_token.clone(),
                resp.refresh_token.clone(),
                resp.expires_in + now,
            )
            .await;

        Ok(())
    }

    pub(crate) async fn keep_refresh_token(api: PixivApi) {
        loop {
            let expire_at = api.inner.read().await.oauth_token.expires_at();
            let now = chrono::Utc::now().timestamp();
            let expire_in = (expire_at - now - 60).max(0); // Refresh 1 minute before expire.

            tokio::time::sleep(tokio::time::Duration::from_secs(expire_in as u64)).await;

            let _ = api.refresh_token().await;
        }
    }
}
