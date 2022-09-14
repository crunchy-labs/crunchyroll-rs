use crate::common::{CrappyBulkResult, FromId};
use crate::error::{CrunchyrollError, CrunchyrollErrorContext};
use crate::media::{Collection, MediaType, Panel};
use crate::search::browse::{BrowseOptions, BrowseSortType};
use crate::{options, BulkResult, Crunchyroll, Executor, Request, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Default, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct CuratedFeed {
    pub id: String,
    pub channel_id: String,

    pub title: String,
    pub description: String,

    pub items: Vec<Panel>,

    #[cfg(feature = "__test_strict")]
    feed_type: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    version: crate::StrictValue,
}

#[async_trait::async_trait]
impl FromId for CuratedFeed {
    async fn from_id(crunchy: &Crunchyroll, id: String) -> Result<Self> {
        let endpoint = format!(
            "https://beta.crunchyroll.com/content/v1/curated_feeds/{}",
            id
        );
        let builder = crunchy
            .executor
            .client
            .get(endpoint)
            .query(&[("locale", &crunchy.executor.details.locale)]);
        crunchy.executor.request(builder).await
    }
}

#[derive(Debug, Deserialize, Default)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct CarouselFeedImages {
    pub landscape_poster: String,
    pub portrait_poster: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct CarouselFeed {
    pub id: u32,
    pub link: String,

    pub slug: String,
    pub title: String,
    pub description: String,

    pub button_text: String,

    pub images: CarouselFeedImages,

    #[cfg(feature = "__test_strict")]
    third_party_impression_tracker: crate::StrictValue,
}

impl CarouselFeed {
    pub async fn collection_from_id(
        crunchy: &Crunchyroll,
        id: String,
    ) -> Result<BulkResult<CarouselFeed>> {
        let endpoint = format!("https://beta.crunchyroll.com/content/v1/carousel/{}", id);
        let builder = crunchy
            .executor
            .client
            .get(endpoint)
            .query(&[("locale", &crunchy.executor.details.locale)]);
        crunchy.executor.request(builder).await
    }
}

pub enum HomeFeedType {
    /// The feed at the top of the Crunchyroll website. Call [`CarouselFeed::collection_from_id`]
    /// with the value of this field to get a collection of usable [`CarouselFeed`] structs.
    CarouselFeed(String),
    /// Represents a series. Call [`Series::from_id`] with the value of this field to get a usable
    /// [`Series`] struct.
    Series(String),
    SimilarTo(String),
    /// Represents a separate feed. Call [`CuratedFeed::from_id`] with the value of this field to
    /// get a usable [`CuratedFeed`] struct.
    CuratedFeed(String),
    /// Recommendations for you. Use [`Crunchyroll::recommendations`] to get them.
    Recommendation,
    /// A episode to continue watching. Use [`Crunchyroll::up_next`] to get it.
    UpNext,
    /// Your watchlist. Use [`Crunchyroll::watchlist`] to get it.
    Watchlist,
    /// News feed. Use [`Crunchyroll::news_feed`] to get it.
    NewsFeed,
    /// Browse content. Use [`Crunchyroll::browse`] with the value of this field as argument. Do not
    /// overwrite [`BrowseOptions::sort`] and [`BrowseOptions::media_type`], this might cause
    /// confusing results.
    Browse(BrowseOptions),
    /// Banner with a link to a Crunchyroll series. Use the first value of this field to get the
    /// series link and the second to get image links. Note that no id is provided (for whatever
    /// reason), so you cannot use [`Series::from_id`] to get the series as api element.
    Banner(String, HomeFeedBannerImages),
}

#[derive(Clone, Debug, Deserialize, Default)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct HomeFeedBannerImages {
    pub mobile_small: String,
    pub mobile_large: String,
    pub desktop_small: String,
    pub desktop_large: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct HomeFeed {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub id: String,
    #[serde(deserialize_with = "crate::internal::serde::deserialize_maybe_none_to_option")]
    pub source_media_id: Option<String>,

    pub title: String,
    #[serde(deserialize_with = "crate::internal::serde::deserialize_maybe_none_to_option")]
    pub source_media_title: Option<String>,
    pub description: String,

    pub display_type: String,
    pub resource_type: String,

    #[serde(rename = "__links__")]
    #[serde(default)]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_resource")]
    resource: String,

    // only populated if `resource_type` is `in_feed_banner`
    link: Option<String>,
    // only populated if `resource_type` is `in_feed_banner`
    banner_images: Option<HomeFeedBannerImages>,

    #[cfg(feature = "__test_strict")]
    new_window: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    promo_image: Option<crate::StrictValue>,
}

impl HomeFeed {
    pub async fn home_feed_type(&self) -> Result<HomeFeedType> {
        match self.resource_type.as_str() {
            "hero_carousel" => Ok(HomeFeedType::CarouselFeed(
                self.id.clone().split('-').last().unwrap().to_string(),
            )),
            "panel" => Ok(HomeFeedType::Series(self.id.clone())),
            "dynamic_collection" => {
                if (&self.resource).contains("recommendations") {
                    Ok(HomeFeedType::Recommendation)
                } else if (&self.resource).contains("similar_to") {
                    Ok(HomeFeedType::SimilarTo(self.id.clone()))
                } else {
                    Err(CrunchyrollError::Internal(CrunchyrollErrorContext::new(
                        format!("invalid dynamic collection type `{}`", self.resource),
                    )))
                }
            }
            "continue_watching" => Ok(HomeFeedType::UpNext),
            "dynamic_watchlist" => Ok(HomeFeedType::Watchlist),
            "news_feed" => Ok(HomeFeedType::NewsFeed),
            "recent_episodes" => Ok(HomeFeedType::Browse(
                BrowseOptions::default()
                    .sort(BrowseSortType::NewlyAdded)
                    .media_type(MediaType::Custom("episode".to_string())),
            )),
            "in_feed_banner" => Ok(HomeFeedType::Banner(
                self.link.clone().unwrap(),
                self.banner_images.clone().unwrap(),
            )),
            "curated_collection" => Ok(HomeFeedType::CuratedFeed(self.id.clone())),
            _ => Err(CrunchyrollError::Internal(CrunchyrollErrorContext::new(
                format!("invalid resource type `{}`", self.resource_type),
            ))),
        }
    }
}

#[derive(Debug, Deserialize, Default, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct NewsFeedResult {
    pub top_news: BulkResult<NewsFeed>,
    pub latest_news: BulkResult<NewsFeed>,
}

#[derive(Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct NewsFeed {
    pub title: String,
    pub description: String,

    pub creator: String,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub publish_date: DateTime<Utc>,

    #[serde(rename = "image")]
    pub image_link: String,
    #[serde(rename = "link")]
    pub news_link: String,
}

#[derive(Debug, Deserialize, smart_default::SmartDefault)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct UpNextEntry {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub new: bool,
    pub new_content: bool,

    pub playhead: u32,
    pub fully_watched: bool,

    pub panel: Panel,
}

impl Request for UpNextEntry {
    fn __set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor.clone();

        self.panel.__set_executor(executor);
    }
}

options! {
    HomeFeedOptions;
    #[doc = "Limit of results to return."]
    limit(u32, "n") = Some(20),
    #[doc = "Specifies the index from which the entries should be returned."]
    start(u32, "start") = None
}

options! {
    NewsFeedOptions;
    #[doc = "Limit number of top news."]
    top_limit(u32, "top_news_n") = Some(20),
    #[doc = "Limit number of latest news."]
    latest_limit(u32, "latest_news_n") = Some(20)
}

options! {
    RecommendationOptions;
    #[doc = "Limit of results to return."]
    limit(u32, "n") = Some(20)
}

options! {
    UpNextOptions;
    #[doc = "Limit of results to return."]
    limit(u32, "n") = Some(20)
}

impl Crunchyroll {
    pub async fn home_feed(&self, options: HomeFeedOptions) -> Result<CrappyBulkResult<HomeFeed>> {
        let endpoint = format!(
            "https://beta.crunchyroll.com/content/v1/{}/home_feed",
            self.executor.details.account_id
        );
        let builder = self
            .executor
            .client
            .get(endpoint)
            .query(&options.to_query(&[(
                "locale".to_string(),
                self.executor.details.locale.to_string(),
            )]));
        self.executor.request(builder).await
    }

    pub async fn news_feed(&self, options: NewsFeedOptions) -> Result<NewsFeedResult> {
        let endpoint = "https://beta.crunchyroll.com/content/v1/news_feed";
        let builder = self
            .executor
            .client
            .get(endpoint)
            .query(&options.to_query(&[(
                "locale".to_string(),
                self.executor.details.locale.to_string(),
            )]));
        self.executor.request(builder).await
    }

    pub async fn recommendations(
        &self,
        options: RecommendationOptions,
    ) -> Result<BulkResult<Collection>> {
        let endpoint = format!(
            "https://beta.crunchyroll.com/content/v1/{}/recommendations",
            self.executor.details.account_id
        );
        let builder = self
            .executor
            .client
            .get(endpoint)
            .query(&options.to_query(&[(
                "locale".to_string(),
                self.executor.details.locale.to_string(),
            )]));
        self.executor.request(builder).await
    }

    pub async fn up_next(&self, options: UpNextOptions) -> Result<BulkResult<UpNextEntry>> {
        let endpoint = format!(
            "	https://beta.crunchyroll.com/content/v1/{}/up_next_account",
            self.executor.details.account_id
        );
        let builder = self
            .executor
            .client
            .get(endpoint)
            .query(&options.to_query(&[(
                "locale".to_string(),
                self.executor.details.locale.to_string(),
            )]));
        self.executor.request(builder).await
    }
}
