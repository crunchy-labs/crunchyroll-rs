use anyhow::Result;
use crunchyroll_rs::Crunchyroll;
use crunchyroll_rs::categories::Category;
use crunchyroll_rs::common::StreamExt;
use crunchyroll_rs::crunchyroll::DeviceIdentifier;
use crunchyroll_rs::search::{BrowseOptions, SearchMediaCollection};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let email = env::var("EMAIL").expect("'EMAIL' environment variable not found");
    let password = env::var("PASSWORD").expect("'PASSWORD' environment variable not found");

    let crunchyroll = Crunchyroll::builder()
        .login_with_credentials(email, password, DeviceIdentifier::default())
        .await?;

    let options = BrowseOptions::default()
        // only dubbed results
        .is_dubbed(true)
        // only results which have action as a category / genre
        .categories(vec![Category::Action]);

    let mut browse_result = crunchyroll.browse(options.clone());
    while let Some(item) = browse_result.next().await {
        match item? {
            SearchMediaCollection::Series(series) => {
                println!("Browse returned series {}", series.title)
            }
            // is never season
            SearchMediaCollection::Episode(episode) => {
                println!("Browse returned episode {}", episode.title)
            }
            SearchMediaCollection::MovieListing(movie_listing) => {
                println!("Browse returned movie listing {}", movie_listing.title)
            }
            SearchMediaCollection::MusicVideo(music_video) => {
                println!("Browse returned music video {}", music_video.title)
            }
            SearchMediaCollection::Concert(concert) => {
                println!("Browse returned concert {}", concert.title)
            }
        }
    }

    Ok(())
}
