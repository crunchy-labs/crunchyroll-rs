#![cfg(feature = "hls-stream")]

use anyhow::Result;
use crunchyroll_rs::{Crunchyroll, Episode};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let user = env::var("USER").expect("'USER' environment variable not found");
    let password = env::var("PASSWORD").expect("'PASSWORD' environment variable not found");

    let crunchyroll = Crunchyroll::builder()
        .login_with_credentials(user, password)
        .await?;

    let episode: Episode = crunchyroll.media_from_id("GRDKJZ81Y").await?;
    let streams = episode.stream().await?;
    let mut default_streams = streams.hls_streaming_data(None).await?;
    // sort after resolutions; best to worst
    default_streams.sort_by(|a, b| a.resolution.width.cmp(&b.resolution.width).reverse());

    // get video segments of the stream with the best available resolution
    let segments = default_streams[0].segments().await?;

    let sink = &mut std::io::sink();
    for (i, segment) in segments.iter().enumerate() {
        println!("Writing segment {} of {}", i + 1, segments.len() + 1);
        segment.clone().write_to(sink).await?;
    }

    Ok(())
}
