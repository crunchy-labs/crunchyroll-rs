#![cfg(feature = "parse")]

use crunchyroll_rs::UrlType;

mod utils;

#[test]
fn parse_series_url() {
    let url = "https://www.crunchyroll.com/series/GY8VEQ95Y/darling-in-the-franxx";
    let parsed = crunchyroll_rs::parse_url(url);

    assert!(matches!(parsed, Some(UrlType::Series { .. })));
    let UrlType::Series(id) = parsed.unwrap() else {
        unreachable!()
    };
    assert_eq!(id, "GY8VEQ95Y");
}

#[test]
fn parse_episode_url() {
    let url = "https://www.crunchyroll.com/de/watch/GRDQPM1ZY/alone-and-lonesome";
    let parsed = crunchyroll_rs::parse_url(url);

    assert!(matches!(parsed, Some(UrlType::EpisodeOrMovie { .. })));
    let UrlType::EpisodeOrMovie(id) = parsed.unwrap() else {
        unreachable!()
    };
    assert_eq!(id, "GRDQPM1ZY")
}

#[test]
fn parse_movie_url() {
    let url = "https://www.crunchyroll.com/watch/G62PEZ2E6/garakowa-restore-the-world-";
    let parsed = crunchyroll_rs::parse_url(url);

    assert!(matches!(parsed, Some(UrlType::EpisodeOrMovie { .. })));
    let UrlType::EpisodeOrMovie(id) = parsed.unwrap() else {
        unreachable!()
    };
    assert_eq!(id, "G62PEZ2E6")
}

#[test]
fn parse_music_video_url() {
    let url = "https://www.crunchyroll.com/de/watch/musicvideo/MV2FD1FECE/gurenge";
    let parsed = crunchyroll_rs::parse_url(url);

    assert!(matches!(parsed, Some(UrlType::MusicVideo { .. })));
    let UrlType::MusicVideo(id) = parsed.unwrap() else {
        unreachable!()
    };
    assert_eq!(id, "MV2FD1FECE")
}

#[test]
fn parse_concert_url() {
    let url = "https://www.crunchyroll.com/watch/concert/MC2E2AC135/live-is-smile-always-364joker-at-yokohama-arena";
    let parsed = crunchyroll_rs::parse_url(url);

    assert!(matches!(parsed, Some(UrlType::Concert { .. })));
    let UrlType::Concert(id) = parsed.unwrap() else {
        unreachable!()
    };
    assert_eq!(id, "MC2E2AC135")
}

#[test]
fn parse_artist_url() {
    let url = "https://www.crunchyroll.com/artist/MA179CB50D/lisa";
    let parsed = crunchyroll_rs::parse_url(url);

    assert!(matches!(parsed, Some(UrlType::Artist { .. })));
    let UrlType::Artist(id) = parsed.unwrap() else {
        unreachable!()
    };
    assert_eq!(id, "MA179CB50D")
}
