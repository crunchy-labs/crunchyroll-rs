use anyhow::Result;
use crunchyroll_rs::categories::Category;
use crunchyroll_rs::search::BrowseOptions;
use crunchyroll_rs::{Crunchyroll, MediaCollection};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let user = env::var("USER").expect("'USER' environment variable not found");
    let password = env::var("PASSWORD").expect("'PASSWORD' environment variable not found");

    let crunchyroll = Crunchyroll::builder()
        .login_with_credentials(user, password)
        .await?;

    let options = BrowseOptions::default()
        // only dubbed results
        .is_dubbed(true)
        // only results which have action as a category / genre
        .categories(vec![Category::Action]);
    let result = crunchyroll.browse(options).await?;

    for item in result.data {
        match item {
            MediaCollection::Series(series) => println!("Browse returned series {}", series.title),
            // is never season
            MediaCollection::Season(_) => (),
            MediaCollection::Episode(episode) => {
                println!("Browse returned episode {}", episode.title)
            }
            MediaCollection::MovieListing(movie_listing) => {
                println!("Browse returned movie listing {}", movie_listing.title)
            }
            MediaCollection::Movie(movie) => println!("Browse returned movie {}", movie.title),
        }
    }

    Ok(())
}
