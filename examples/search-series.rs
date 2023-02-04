use anyhow::Result;
use crunchyroll_rs::Crunchyroll;
use futures_util::StreamExt;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let user = env::var("USER").expect("'USER' environment variable not found");
    let password = env::var("PASSWORD").expect("'PASSWORD' environment variable not found");

    let crunchyroll = Crunchyroll::builder()
        .login_with_credentials(user, password)
        .await?;

    while let Some(s) = crunchyroll.query("darling").series.next().await {
        let series = s?;

        println!(
            "Queried series {} which has {} seasons",
            series.title, series.season_count
        );
        let seasons = series.seasons().await?;
        for season in seasons {
            println!(
                "Found season {} with audio locale(s) {}",
                season.season_number,
                season
                    .audio_locales
                    .iter()
                    .map(|l| l.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        }
    }

    Ok(())
}
