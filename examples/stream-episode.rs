#![cfg(feature = "stream")]

use anyhow::Result;
use crunchyroll::Crunchyroll;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let user = env::var("USER").expect("'USER' environment variable not found");
    let password = env::var("PASSWORD").expect("'PASSWORD' environment variable not found");

    let crunchyroll = Crunchyroll::builder()
        .login_with_credentials(user, password)
        .await?;

    let episode = crunchyroll.episode_from_id("GRDKJZ81Y".into()).await?;
    let streams = episode.streams().await?;
    let mut default_streams = streams.streaming_data().await?;
    // sort after resolutions; best to worst
    default_streams.sort_by(|a, b| a.resolution.width.cmp(&b.resolution.width).reverse());

    // get video segments of the stream with the best available resolution
    let segments = default_streams[0].segments().await?;

    let sink = &mut std::io::sink();
    for segment in segments {
        segment.write_to(sink).await?;
    }

    Ok(())
}
