#![cfg(feature = "parse")]

use crunchyroll_rs::UrlType;

mod utils;

#[test]
fn parse_series_url() {
    let url = "https://www.crunchyroll.com/de/series/GY8VEQ95Y/darling-in-the-franxx";
    let parsed = crunchyroll_rs::parse_url(url);

    assert!(parsed.is_some());
    assert!(matches!(parsed.clone().unwrap(), UrlType::Series { .. }));
    if let UrlType::Series(id) = parsed.unwrap() {
        assert_eq!(id, "GY8VEQ95Y")
    } else {
        unreachable!()
    }
}

#[test]
fn parse_episode_url() {
    let url = "https://www.crunchyroll.com/de/watch/GRDQPM1ZY/alone-and-lonesome";
    let parsed = crunchyroll_rs::parse_url(url);

    assert!(parsed.is_some());
    assert!(matches!(
        parsed.clone().unwrap(),
        UrlType::EpisodeOrMovie { .. }
    ));
    if let UrlType::EpisodeOrMovie(id) = parsed.unwrap() {
        assert_eq!(id, "GRDQPM1ZY")
    } else {
        unreachable!()
    }
}

#[test]
fn parse_movie_url() {
    let url = "https://www.crunchyroll.com/de/watch/G62PEZ2E6/garakowa-restore-the-world-";
    let parsed = crunchyroll_rs::parse_url(url);

    assert!(parsed.is_some());
    assert!(matches!(
        parsed.clone().unwrap(),
        UrlType::EpisodeOrMovie { .. }
    ));
    if let UrlType::EpisodeOrMovie(id) = parsed.unwrap() {
        assert_eq!(id, "G62PEZ2E6")
    } else {
        unreachable!()
    }
}

#[test]
fn parse_music_video_url() {
    let url = "https://www.crunchyroll.com/de/watch/musicvideo/MV2FD1FECE/gurenge";
    let parsed = crunchyroll_rs::parse_url(url);

    assert!(parsed.is_some());
    assert!(matches!(
        parsed.clone().unwrap(),
        UrlType::MusicVideo { .. }
    ));
    if let UrlType::MusicVideo(id) = parsed.unwrap() {
        assert_eq!(id, "MV2FD1FECE")
    } else {
        unreachable!()
    }
}

#[test]
fn parse_concert_url() {
    let url = "https://www.crunchyroll.com/de/watch/concert/MC2E2AC135/live-is-smile-always-364joker-at-yokohama-arena";
    let parsed = crunchyroll_rs::parse_url(url);

    assert!(parsed.is_some());
    assert!(matches!(parsed.clone().unwrap(), UrlType::Concert { .. }));
    if let UrlType::Concert(id) = parsed.unwrap() {
        assert_eq!(id, "MC2E2AC135")
    } else {
        unreachable!()
    }
}
