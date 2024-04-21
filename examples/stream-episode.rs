use anyhow::Result;
use crunchyroll_rs::{Crunchyroll, Episode};
use std::env;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    let email = env::var("EMAIL").expect("'EMAIL' environment variable not found");
    let password = env::var("PASSWORD").expect("'PASSWORD' environment variable not found");

    let crunchyroll = Crunchyroll::builder()
        .login_with_credentials(email, password)
        .await?;

    let episode: Episode = crunchyroll.media_from_id("GRDKJZ81Y").await?;
    let stream = episode.stream_maybe_without_drm().await?;
    let (mut video_streams, _audio_streams) = stream.stream_data(None).await?.unwrap();
    // sort after resolutions; best to worst
    video_streams.sort_by(|a, b| {
        a.resolution()
            .unwrap()
            .width
            .cmp(&b.resolution().unwrap().width)
            .reverse()
    });

    // get video segments of the stream with the best available resolution
    let segments = video_streams[0].segments();

    let sink = &mut std::io::sink();
    for (i, segment) in segments.iter().enumerate() {
        println!("Downloading segment {} of {}", i + 1, segments.len() + 1);
        sink.write_all(&segment.data().await?)?;
    }

    Ok(())
}
