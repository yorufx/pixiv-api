use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{error::Result, model::Illust, Error, PixivApi};

const API_V1_URL: &str = "https://app-api.pixiv.net/v1";

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[error("{:?}", self)]
pub struct PixivError {
    user_message: Option<String>,
    message: Option<String>,
    reason: Option<String>,
    user_message_details: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PixivResponse<T> {
    Error { error: PixivError },
    Success(T),
}

impl<T> PixivResponse<T> {
    pub fn ok(self) -> Result<T> {
        match self {
            PixivResponse::Error { error } => Err(Error::Pixiv(error)),
            PixivResponse::Success(a) => Ok(a),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RankingMode {
    Day,
    DayMale,
    DayFemale,
    WeekOriginal,
    WeekRookie,
    Week,
    Month,
    DayR18,
    DayMaleR18,
    WeekR18,
    #[serde(rename = "week_r18g")]
    WeekR18G,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IllustRankingRequest {
    mode: RankingMode,
    filter: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IllustRankingResponse {
    illusts: Vec<Illust>,
    next_url: Option<String>,
}

impl IllustRankingRequest {
    fn new(mode: RankingMode) -> Self {
        Self {
            mode,
            filter: "for_ios",
        }
    }
}

impl PixivApi {
    pub async fn illust_ranking(&self, mode: RankingMode) -> Result<Vec<Illust>> {
        let url = format!("{}/illust/ranking", API_V1_URL);

        let query = IllustRankingRequest::new(mode);
        let resp = self
            .inner
            .read()
            .await
            .client
            .get(url)
            .query(&query)
            .send()
            .await?;

        let resp: PixivResponse<IllustRankingResponse> = resp.json().await?;
        Ok(resp.ok()?.illusts)
    }
}
