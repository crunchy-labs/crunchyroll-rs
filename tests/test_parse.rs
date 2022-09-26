#![cfg(feature = "parse")]

use crunchyroll::parse::UrlType;
use crunchyroll::Crunchyroll;

mod utils;

#[test]
fn parse_beta_series_url() {
    let url = "https://beta.crunchyroll.com/de/series/GY8VEQ95Y/darling-in-the-franxx";
    let parsed = Crunchyroll::parse_url(url);

    assert_result!(parsed);
    assert!(matches!(parsed.unwrap(), UrlType::BetaSeries { .. }))
}

#[test]
fn parse_beta_episode_url() {
    let url = "https://beta.crunchyroll.com/de/watch/GRDQPM1ZY/alone-and-lonesome";
    let parsed = Crunchyroll::parse_url(url);

    assert_result!(parsed);
    assert!(matches!(
        parsed.unwrap(),
        UrlType::BetaEpisodeOrMovie { .. }
    ))
}

#[test]
fn parse_beta_movie_url() {
    let url = "https://beta.crunchyroll.com/de/watch/G62PEZ2E6/garakowa-restore-the-world-";
    let parsed = Crunchyroll::parse_url(url);

    assert_result!(parsed);
    assert!(matches!(
        parsed.unwrap(),
        UrlType::BetaEpisodeOrMovie { .. }
    ))
}

#[test]
fn parse_classic_series_url() {
    let url = "https://www.crunchyroll.com/darling-in-the-franxx";
    let parsed = Crunchyroll::parse_url(url);

    assert_result!(parsed);
    assert!(matches!(
        parsed.unwrap(),
        UrlType::ClassicSeriesOrMovieListing { .. }
    ))
}

#[test]
fn parse_classic_movie_listing_url() {
    let url = "https://www.crunchyroll.com/garakowa-restore-the-world-";
    let parsed = Crunchyroll::parse_url(url);

    assert_result!(parsed);
    assert!(matches!(
        parsed.unwrap(),
        UrlType::ClassicSeriesOrMovieListing { .. }
    ))
}

#[test]
fn parse_classic_episode_url() {
    let url =
        "https://www.crunchyroll.com/darling-in-the-franxx/episode-1-alone-and-lonesome-759575";
    let parsed = Crunchyroll::parse_url(url);

    assert_result!(parsed);
    assert!(matches!(parsed.unwrap(), UrlType::ClassicEpisode { .. }))
}

#[test]
fn parse_classic_movie_url() {
    let url = "https://www.crunchyroll.com/garakowa-restore-the-world-/garakowa-restore-the-world-movie-693261";
    let parsed = Crunchyroll::parse_url(url);

    assert_result!(parsed);
    assert!(matches!(parsed.unwrap(), UrlType::ClassicMovie { .. }))
}
