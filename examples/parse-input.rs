#![cfg(feature = "parse")]

use anyhow::Result;
use crunchyroll_rs::parse::UrlType;
use crunchyroll_rs::Crunchyroll;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let url = env::var("URL").expect(
        "please set the 'URL' environment variable to any crunchyroll url which points to a media",
    );

    let parsed = Crunchyroll::parse_url(url)?;
    match parsed {
        UrlType::BetaSeries(_) => println!("url points to a curnchyroll beta series"),
        UrlType::BetaMovieListing(_) => println!("url points to a crunchyroll beta movie listing"),
        UrlType::BetaEpisodeOrMovie(_) => {
            println!("url points to a crunchyroll beta episode or movie")
        }
        UrlType::ClassicSeriesOrMovieListing(_) => {
            println!("url points to a crunchyroll classic series or movie listing")
        }
        UrlType::ClassicEpisode { .. } => println!("url points to a crunchyroll classic episode"),
        UrlType::ClassicMovie { .. } => println!("url points to a crunchyroll classic movie"),
    }

    Ok(())
}
