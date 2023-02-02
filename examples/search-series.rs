use anyhow::Result;
use crunchyroll_rs::search::QueryOptions;
use crunchyroll_rs::Crunchyroll;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let user = env::var("USER").expect("'USER' environment variable not found");
    let password = env::var("PASSWORD").expect("'PASSWORD' environment variable not found");

    let crunchyroll = Crunchyroll::builder()
        .login_with_credentials(user, password)
        .await?;

    let options = QueryOptions::default()
        // return 2o items max
        .limit(20);
    let result = crunchyroll.query("darling", options).await?;

    let series = result.series.unwrap();
    for s in series.items {
        println!(
            "Queried series {} which has {} seasons",
            s.title, s.season_count
        );
        let seasons = s.seasons(None).await?;
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
