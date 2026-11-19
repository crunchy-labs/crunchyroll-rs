use crunchyroll_rs::Crunchyroll;
use crunchyroll_rs::auth::DeviceIdentifier;
use crunchyroll_rs::categories::Category;
use crunchyroll_rs::common::StreamExt;
use crunchyroll_rs::search::{BrowseOptions, SearchMediaCollection, SearchMusicType};
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let etp_rt = env::var("ETP_RT").expect("'ETP_RT' environment variable not found");

    let crunchyroll = Crunchyroll::builder()
        .login_with_etp_rt(etp_rt, DeviceIdentifier::default())
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
            SearchMediaCollection::Movie(movie) => {
                println!("Browse returned movie listing {}", movie.title)
            }
            SearchMediaCollection::Music(music) => match music.music_type {
                SearchMusicType::MusicVideo(music_video) => {
                    println!("Browse returned music video {}", music_video.title)
                }
                SearchMusicType::Concert(concert) => {
                    println!("Browse returned concert {}", concert.title)
                }
            },
        }
    }

    Ok(())
}
