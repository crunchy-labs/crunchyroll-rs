#![cfg(feature = "parse")]

use anyhow::Result;
use crunchyroll_rs::parse::UrlType;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let url = env::var("URL").expect(
        "please set the 'URL' environment variable to any crunchyroll url which points to a media",
    );

    let parsed = crunchyroll_rs::parse_url(url).expect("url is not valid");
    match parsed {
        UrlType::Series(_) => println!("url points to a crunchyroll series"),
        UrlType::MovieListing(_) => println!("url points to a crunchyroll movie listing"),
        UrlType::EpisodeOrMovie(_) => println!("url points to a crunchyroll episode or movie"),
    }

    Ok(())
}
