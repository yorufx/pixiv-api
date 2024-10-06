use serde::{Deserialize, Serialize};

const PIXIV_HOST: &str = "https://pixiv.net";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Illust {
    pub id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub height: u32,
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub tags: Vec<Tag>,

    #[serde(default)]
    pub page_count: u32,
    #[serde(default)]
    image_urls: ImageUrls,
    #[serde(default)]
    meta_pages: Vec<MetaPage>,
    #[serde(default)]
    meta_single_page: MetaSinglePage,

    #[serde(default)]
    pub total_bookmarks: u64,
    #[serde(default)]
    pub total_view: u64,

    pub user: User,
}

impl Illust {
    pub fn images(&self) -> Vec<ImageUrls> {
        if !self.meta_pages.is_empty() {
            self.meta_pages
                .iter()
                .map(|page| page.image_urls.clone())
                .collect()
        } else {
            vec![ImageUrls {
                square_medium: self.image_urls.square_medium.clone(),
                medium: self.image_urls.medium.clone(),
                large: self.image_urls.large.clone(),
                original: self.meta_single_page.original_image_url.clone(),
            }]
        }
    }

    pub fn preview_url(&self) -> Option<&str> {
        self.image_urls.medium.as_deref()
    }

    pub fn url(&self) -> String {
        format!("{}/artworks/{}", PIXIV_HOST, self.id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub translated_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub account: String,
    pub name: String,
}

impl User {
    pub fn url(&self) -> String {
        format!("{}/users/{}", PIXIV_HOST, self.id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaPage {
    pub image_urls: ImageUrls,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetaSinglePage {
    pub original_image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageUrls {
    pub square_medium: Option<String>,
    pub medium: Option<String>,
    pub large: Option<String>,
    pub original: Option<String>,
}
