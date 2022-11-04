#![cfg(feature = "parse")]

use crunchyroll_rs::UrlType;

mod utils;

#[test]
fn parse_series_url() {
    let url = "https://www.crunchyroll.com/de/series/GY8VEQ95Y/darling-in-the-franxx";
    let parsed = crunchyroll_rs::parse_url(url);

    assert!(parsed.is_some());
    assert!(matches!(parsed.unwrap(), UrlType::Series { .. }))
}

#[test]
fn parse_episode_url() {
    let url = "https://www.crunchyroll.com/de/watch/GRDQPM1ZY/alone-and-lonesome";
    let parsed = crunchyroll_rs::parse_url(url);

    assert!(parsed.is_some());
    assert!(matches!(parsed.unwrap(), UrlType::EpisodeOrMovie { .. }))
}

#[test]
fn parse_movie_url() {
    let url = "https://www.crunchyroll.com/de/watch/G62PEZ2E6/garakowa-restore-the-world-";
    let parsed = crunchyroll_rs::parse_url(url);

    assert!(parsed.is_some());
    assert!(matches!(parsed.unwrap(), UrlType::EpisodeOrMovie { .. }))
}
