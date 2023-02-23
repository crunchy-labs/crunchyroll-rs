use crate::common::{Pagination, V2BulkResult, V2TypeBulkResult};
use crate::media::MediaType;
use crate::search::{BrowseOptions, BrowseSortType};
use crate::{Crunchyroll, MediaCollection, Request, Series};
use chrono::{DateTime, Utc};
use futures_util::FutureExt;
use serde::de::Error;
use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, Default, Deserialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct FeedCarouselImages {
    pub landscape_poster: Option<String>,
    #[serde(alias = "portrait_image")]
    pub portrait_poster: Option<String>,
}

/// The carousel / sliding images showed at first when visiting crunchyroll.com
#[allow(dead_code)]
#[derive(Clone, Debug, Default, Deserialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct FeedCarousel {
    pub title: String,
    pub slug: String,
    pub description: String,

    /// Link to a crunchyroll series or article.
    pub link: String,

    pub images: FeedCarouselImages,

    pub button_text: String,

    #[cfg(feature = "__test_strict")]
    id: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    third_party_impression_tracker: crate::StrictValue,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct FeedBannerImages {
    pub mobile_small: String,
    pub mobile_large: String,
    pub desktop_small: String,
    pub desktop_large: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FeedBanner {
    pub title: String,

    pub description: String,

    /// Link to a crunchyroll series or article.
    pub link: String,

    pub images: FeedBannerImages,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SeriesFeed {
    pub title: String,

    pub description: String,

    /// Ids to series. Use [`Series::from_id`] to get the series.
    pub ids: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SimilarFeed {
    pub title: String,

    pub description: String,

    #[serde(skip)]
    pub similar_id: String,
}

#[derive(Clone, Debug, Request)]
pub enum HomeFeed {
    /// The feed at the top of the Crunchyroll website.
    CarouselFeed(Vec<FeedCarousel>),
    /// A series recommendation.
    Series(Series),
    /// Recommendations for you. Use [`Crunchyroll::recommendations`] to get them.
    Recommendation,
    /// Your watch history. Use [`Crunchyroll::watch_history`] to get it.
    History,
    /// A banner containing a link to a series or article.
    Banner(FeedBanner),
    /// Your watchlist. Use [`Crunchyroll::watchlist`] to get it.
    Watchlist,
    /// A feed containing a title with description and multiple series (ids) matching to title and
    /// description.
    SeriesFeed(SeriesFeed),
    /// A feed containing ids to music videos. Use [`crate::MusicVideo::from_id`] to get usable
    /// structs from it.
    MusicVideoFeed(Vec<String>),
    /// A feed containing ids to concerts. Use [`crate::Concert::from_id`] to get usable structs from
    /// it.
    ConcertFeed(Vec<String>),
    /// News feed. Use [`Crunchyroll::news_feed`] to get it.
    NewsFeed,
    /// Browse content. Use [`Crunchyroll::browse`] with the value of this field as argument. Do not
    /// overwrite [`BrowseOptions::sort`] and [`BrowseOptions::media_type`], this might cause
    /// confusing results.
    Browse(BrowseOptions),
    /// Results similar to a series. Get the series struct via [`SimilarFeed::similar_id`] and call
    /// [`Series::similar`] to get similar series.
    SimilarTo(SimilarFeed),
    /// Crunchyroll may update their feed / add new items. This field catches everything which is
    /// unknown / not implemented in the library.
    Unknown(serde_json::Map<String, serde_json::Value>),
}

impl Default for HomeFeed {
    fn default() -> Self {
        Self::Unknown(serde_json::Map::default())
    }
}

impl<'de> Deserialize<'de> for HomeFeed {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut as_map = serde_json::Map::deserialize(deserializer)?;

        let type_error = |k: &str, t: &str| Error::custom(format!("home feed '{k}' is no {t}"));
        let mut get_value = |k: &str| {
            as_map
                .remove(k)
                .ok_or_else(|| Error::custom(format!("cannot get '{k}' on home feed")))
        };
        let map_serde_error = |e: serde_json::Error| Error::custom(e.to_string());

        let resource_type = get_value("resource_type")?
            .as_str()
            .ok_or_else(|| type_error("resource_type", "string"))?
            .to_string();

        match resource_type.as_str() {
            "hero_carousel" => Ok(Self::CarouselFeed(
                serde_json::from_value(get_value("items")?).map_err(map_serde_error)?,
            )),
            "panel" => Ok(Self::Series(
                serde_json::from_value(get_value("panel")?).map_err(map_serde_error)?,
            )),
            "dynamic_collection" => {
                let response_type = get_value("response_type")?
                    .as_str()
                    .ok_or_else(|| type_error("response_type", "string"))?
                    .to_string();

                match response_type.as_str() {
                    "recommendations" => Ok(Self::Recommendation),
                    "history" => Ok(Self::History),
                    "watchlist" => Ok(Self::Watchlist),
                    "news_feed" => Ok(Self::NewsFeed),
                    "browse" | "recent_episodes" => {
                        let link = get_value("link")?
                            .as_str()
                            .ok_or_else(|| type_error("link", "string"))?
                            .to_string();
                        let query: Vec<(String, String)> =
                            serde_urlencoded::from_str(link.split('?').last().unwrap())
                                .map_err(|e| Error::custom(e.to_string()))?;

                        let mut browse_options = BrowseOptions::default();
                        for (key, value) in query {
                            match key.as_str() {
                                "sort_by" => {
                                    browse_options =
                                        browse_options.sort(BrowseSortType::from(value))
                                }
                                "type" => {
                                    browse_options =
                                        browse_options.media_type(MediaType::from(value))
                                }
                                _ => (),
                            }
                        }

                        Ok(Self::Browse(browse_options))
                    }
                    "because_you_watched" => {
                        let id = get_value("source_media_id")?
                            .as_str()
                            .ok_or_else(|| type_error("source_media_id", "string"))?
                            .to_string();

                        let mut similar_feed: SimilarFeed = serde_json::from_value(
                            serde_json::to_value(as_map).map_err(map_serde_error)?,
                        )
                        .map_err(map_serde_error)?;
                        similar_feed.similar_id = id;

                        Ok(Self::SimilarTo(similar_feed))
                    }
                    _ => Err(Error::custom(format!(
                        "cannot parse home feed response type '{response_type}'"
                    ))),
                }
            }
            "in_feed_banner" => Ok(Self::Banner(
                serde_json::from_value(serde_json::to_value(as_map).map_err(map_serde_error)?)
                    .map_err(map_serde_error)?,
            )),
            "curated_collection" => {
                let response_type = get_value("response_type")?
                    .as_str()
                    .ok_or_else(|| type_error("response_type", "string"))?
                    .to_string();

                match response_type.as_str() {
                    "series" => Ok(Self::SeriesFeed(
                        serde_json::from_value(
                            serde_json::to_value(as_map).map_err(map_serde_error)?,
                        )
                        .map_err(map_serde_error)?,
                    )),
                    "music_concert" => {
                        let ids: Vec<String> = get_value("ids")?
                            .as_array()
                            .ok_or_else(|| type_error("ids", "string list"))?
                            .iter()
                            .map(|v| v.to_string())
                            .collect();
                        Ok(Self::ConcertFeed(ids))
                    }
                    "music_video" => {
                        let ids: Vec<String> = get_value("ids")?
                            .as_array()
                            .ok_or_else(|| type_error("ids", "string list"))?
                            .iter()
                            .map(|v| v.to_string())
                            .collect();
                        Ok(Self::MusicVideoFeed(ids))
                    }
                    #[cfg(feature = "__test_strict")]
                    _ => Err(Error::custom(format!(
                        "cannot parse home feed response type '{response_type}'"
                    ))),
                    #[cfg(not(feature = "__test_strict"))]
                    _ => Ok(HomeFeed::Unknown(as_map)),
                }
            }
            #[cfg(feature = "__test_strict")]
            _ => Err(Error::custom(format!(
                "cannot parse home feed resource type '{}' ({})",
                resource_type,
                serde_json::to_value(&as_map).unwrap()
            ))),
            #[cfg(not(feature = "__test_strict"))]
            _ => Ok(HomeFeed::Unknown(as_map)),
        }
    }
}

pub struct NewsFeedResult {
    pub top_news: Pagination<NewsFeed>,
    pub latest_news: Pagination<NewsFeed>,
}

/// Crunchyroll news like new library anime, dubs, etc... .
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
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

impl Crunchyroll {
    /// Returns the home feed (shown when visiting the Crunchyroll index page).
    pub fn home_feed(&self) -> Pagination<HomeFeed> {
        Pagination::new(
            |options| {
                async move {
                    let endpoint = format!(
                        "https://www.crunchyroll.com/content/v2/discover/{}/home_feed",
                        options.executor.details.account_id.clone()?
                    );
                    let result: V2BulkResult<HomeFeed> = options
                        .executor
                        .get(endpoint)
                        .query(&[("n", options.page_size), ("start", options.start)])
                        .apply_locale_query()
                        .apply_preferred_audio_locale_query()
                        .request()
                        .await?;
                    Ok((result.data, result.total))
                }
                .boxed()
            },
            self.executor.clone(),
            None,
            None,
        )
    }

    /// Returns Crunchyroll news.
    pub fn news_feed(&self) -> NewsFeedResult {
        NewsFeedResult {
            top_news: Pagination::new(
                |options| {
                    async move {
                        let endpoint = "https://www.crunchyroll.com/content/v2/discover/news_feed";
                        let result: V2BulkResult<V2TypeBulkResult<NewsFeed>> = options
                            .executor
                            .get(endpoint)
                            .query(&[("latest_news_n", "0")])
                            .query(&[
                                ("top_news_n", options.page_size),
                                ("top_news_start", options.start),
                            ])
                            .apply_locale_query()
                            .apply_preferred_audio_locale_query()
                            .request()
                            .await?;
                        let top_news = result
                            .data
                            .into_iter()
                            .find(|p| p.result_type == "top_news")
                            .unwrap_or_default();
                        Ok((top_news.items, top_news.total))
                    }
                    .boxed()
                },
                self.executor.clone(),
                None,
                None,
            ),
            latest_news: Pagination::new(
                |options| {
                    async move {
                        let endpoint = "https://www.crunchyroll.com/content/v2/discover/news_feed";
                        let result: V2BulkResult<V2TypeBulkResult<NewsFeed>> = options
                            .executor
                            .get(endpoint)
                            .query(&[("top_news_n", "0")])
                            .query(&[
                                ("latest_news_n", options.page_size),
                                ("latest_news_start", options.start),
                            ])
                            .apply_locale_query()
                            .apply_preferred_audio_locale_query()
                            .request()
                            .await?;
                        let top_news = result
                            .data
                            .into_iter()
                            .find(|p| p.result_type == "top_news")
                            .unwrap_or_default();
                        Ok((top_news.items, top_news.total))
                    }
                    .boxed()
                },
                self.executor.clone(),
                None,
                None,
            ),
        }
    }

    /// Returns recommended series or movies to watch.
    pub fn recommendations(&self) -> Pagination<MediaCollection> {
        Pagination::new(
            |options| {
                async move {
                    let endpoint = format!(
                        "https://www.crunchyroll.com/content/v2/discover/{}/recommendations",
                        options.executor.details.account_id.clone()?
                    );
                    let result: V2BulkResult<MediaCollection> = options
                        .executor
                        .get(endpoint)
                        .query(&[("n", options.page_size), ("start", options.start)])
                        .apply_locale_query()
                        .apply_preferred_audio_locale_query()
                        .request()
                        .await?;
                    Ok((result.data, result.total))
                }
                .boxed()
            },
            self.executor.clone(),
            None,
            None,
        )
    }
}
