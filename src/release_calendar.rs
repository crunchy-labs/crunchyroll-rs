//! Release calendar functionality.
//!
//! Scrapes the public Crunchyroll simulcast calendar page to retrieve upcoming episode releases.

use crate::error::ErrorKind;
use crate::{Crunchyroll, Error, Result, UrlType, parse_url};
use chrono::{DateTime, Utc};
use scraper::{ElementRef, Html, Selector};
use serde::Deserialize;

/// A single scheduled release of a series episode.
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault)]
pub struct ReleaseCalendarItem {
    /// Id of the series the episode belongs to.
    pub series_id: String,
    /// Id of the episode.
    pub episode_id: String,

    /// Title of the season (e.g. series title).
    pub season_title: String,
    /// Title of the episode.
    pub episode_title: String,

    /// Episode number within the season.
    pub episode_number: f32,

    /// UTC timestamp at which the episode is scheduled to be released.
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub release_time: DateTime<Utc>,

    /// Whether a premium account is needed to watch this episode.
    pub premium: bool,
    /// URL of the episode thumbnail image.
    pub thumbnail_url: String,
}

/// Releases of a single week, grouped by weekday.
#[derive(Clone, Debug, Deserialize, Default)]
pub struct ReleaseCalendarWeek {
    pub monday: Vec<ReleaseCalendarItem>,
    pub tuesday: Vec<ReleaseCalendarItem>,
    pub wednesday: Vec<ReleaseCalendarItem>,
    pub thursday: Vec<ReleaseCalendarItem>,
    pub friday: Vec<ReleaseCalendarItem>,
    pub saturday: Vec<ReleaseCalendarItem>,
    pub sunday: Vec<ReleaseCalendarItem>,
}

impl Crunchyroll {
    /// Get the release calendar for the week containing. The release calendar for the whole week is
    /// returned, regardless of the set time or date of `date`.
    ///
    /// Crunchyroll probably only returns the release calendar up until today. In theory Crunchyroll
    /// could also return releases of upcoming days, but until now I only saw one occurrence where
    /// they did that. This occurrence actually the reason for me to implement this endpoint, but
    /// now as I'm writing the comment, they reverted it. Idk why, maybe an intern pushed an
    /// actually useful feature but some senior reverted it because doing something good for the
    /// customer? No, no, not with us.
    ///
    /// *Note*: This might break at any time. This method manually parses the html returned by the
    /// release calendar page, which isn't really resilient against changes.
    pub async fn release_calendar(&self, date: DateTime<Utc>) -> Result<ReleaseCalendarWeek> {
        let url = format!(
            "https://www.crunchyroll.com/simulcastcalendar?filter=premium&date={}",
            date.format("%Y-%m-%d")
        );

        let mut release_calendar_week = ReleaseCalendarWeek::default();

        let calendar_html_raw = self.executor.get(&url).request_raw(false).await?;
        let calendar_html_str = str::from_utf8(&calendar_html_raw).map_err(|e| {
            Error::error_from_other_error_and_url(
                e,
                ErrorKind::Decode {
                    content: Some(calendar_html_raw.clone()),
                },
                &url,
            )
        })?;

        let map_err_fn = |msg: &str| {
            Error::error_from_kind_and_url(
                ErrorKind::Decode {
                    content: Some(calendar_html_raw.clone()),
                },
                &url,
                msg,
            )
        };

        let html = Html::parse_document(calendar_html_str);

        let days_selector = Selector::parse("ol.days").unwrap();
        let days_elem = html
            .select(&days_selector)
            .next()
            .ok_or_else(|| map_err_fn("failed to select ol.days"))?;

        let day_selector = Selector::parse("li.day").unwrap();
        let day_elems = days_elem.select(&day_selector);

        for (i, day_elem) in day_elems.enumerate() {
            let items = match i {
                0 => &mut release_calendar_week.monday,
                1 => &mut release_calendar_week.tuesday,
                2 => &mut release_calendar_week.wednesday,
                3 => &mut release_calendar_week.thursday,
                4 => &mut release_calendar_week.friday,
                5 => &mut release_calendar_week.saturday,
                6 => &mut release_calendar_week.sunday,
                _ => return Err(map_err_fn("got more than 7 weekdays")),
            };

            let release_selector = Selector::parse("ol.releases").unwrap();
            // is none if no releases are present for the day
            let Some(release_elems) = day_elem.select(&release_selector).next() else {
                continue;
            };

            for release_elem in release_elems.child_elements() {
                let season_name_selector = Selector::parse("h1.season-name").unwrap();
                let season_name_elem = release_elem
                    .select(&season_name_selector)
                    .next()
                    .ok_or_else(|| map_err_fn("failed to select episode h1.season-name"))?;

                let episode_article_selector = Selector::parse("article.featured-episode").unwrap();
                let episode_article_elem = release_elem
                    .select(&episode_article_selector)
                    .next()
                    .ok_or_else(|| {
                        map_err_fn("failed to select episode article.featured-episode")
                    })?;

                items.push(ReleaseCalendarItem {
                    series_id: {
                        let series_url = select_html_itemprop(
                            map_err_fn,
                            &season_name_elem,
                            "url",
                            Some("href"),
                        )?;
                        match parse_url(series_url).unwrap() {
                            UrlType::Series(series_id) => series_id,
                            _ => {
                                return Err(map_err_fn(&format!(
                                    "expected series url: {series_url}"
                                )));
                            }
                        }
                    },
                    episode_id: {
                        let episode_url = select_html_itemprop(
                            map_err_fn,
                            &episode_article_elem,
                            "url",
                            Some("href"),
                        )?;
                        match parse_url(episode_url).unwrap() {
                            UrlType::EpisodeOrMovie(episode_id) => episode_id,
                            _ => {
                                return Err(map_err_fn(&format!(
                                    "expected episode url: {episode_url}"
                                )));
                            }
                        }
                    },
                    season_title: select_html_itemprop(
                        map_err_fn,
                        &season_name_elem,
                        "name",
                        None,
                    )?
                    .to_string(),
                    episode_title: select_html_itemprop(
                        map_err_fn,
                        &episode_article_elem,
                        "name",
                        None,
                    )?
                    .to_string(),
                    episode_number: select_html_itemprop(
                        map_err_fn,
                        &episode_article_elem,
                        "episodeNumber",
                        Some("content"),
                    )?
                    .parse()
                    .unwrap(),
                    release_time: {
                        let date_published = select_html_itemprop(
                            map_err_fn,
                            &episode_article_elem,
                            "datePublished",
                            Some("content"),
                        )?;
                        DateTime::parse_from_rfc3339(date_published)
                            .unwrap()
                            .with_timezone(&Utc)
                    },
                    premium: {
                        let premium_selector = Selector::parse("svg.premium-flag").unwrap();
                        episode_article_elem
                            .select(&premium_selector)
                            .next()
                            .is_some()
                    },
                    thumbnail_url: select_html_itemprop(
                        map_err_fn,
                        &episode_article_elem,
                        "image",
                        Some("src"),
                    )?
                    .to_string(),
                });
            }
        }

        Ok(release_calendar_week)
    }
}

fn select_html_itemprop<'a, F: Fn(&str) -> Error>(
    map_err_fn: F,
    elem: &'a ElementRef,
    itemprop: &str,
    attr: Option<&str>,
) -> Result<&'a str> {
    let selector = Selector::parse(&format!("[itemprop={itemprop}]")).unwrap();
    let selected_elem = elem
        .select(&selector)
        .next()
        .ok_or_else(|| map_err_fn(&format!("failed to select episode [itemprop={itemprop}]")))?;
    if let Some(attr) = attr {
        selected_elem.attr(attr).ok_or_else(|| {
            map_err_fn(&format!(
                "failed to select episode [itemprop={itemprop}] attr"
            ))
        })
    } else {
        selected_elem.text().next().ok_or_else(|| {
            map_err_fn(&format!(
                "failed to select episode [itemprop={itemprop}] text content"
            ))
        })
    }
}
