use chrono::{Datelike, Utc, Weekday};
use crunchyroll_rs::Crunchyroll;
use crunchyroll_rs::crunchyroll::DeviceIdentifier;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let email = env::var("EMAIL").expect("'EMAIL' environment variable not found");
    let password = env::var("PASSWORD").expect("'PASSWORD' environment variable not found");

    let crunchyroll = Crunchyroll::builder()
        .login_with_credentials(email, password, DeviceIdentifier::default())
        .await?;

    let now = Utc::now();
    let release_calendar_week = crunchyroll.release_calendar(now).await?;

    let releases_today = match now.weekday() {
        Weekday::Mon => release_calendar_week.monday,
        Weekday::Tue => release_calendar_week.tuesday,
        Weekday::Wed => release_calendar_week.wednesday,
        Weekday::Thu => release_calendar_week.thursday,
        Weekday::Fri => release_calendar_week.friday,
        Weekday::Sat => release_calendar_week.saturday,
        Weekday::Sun => release_calendar_week.sunday,
    };

    println!("Releases today:");
    for release in releases_today {
        println!(
            "  {} => {}; Episode {} ({})",
            release.release_time, release.season_title, release.episode_number, release.episode_title
        )
    }

    Ok(())
}
