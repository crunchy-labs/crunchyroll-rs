use crate::crunchyroll::CrunchyrollBuilder;
use crate::Result;
use http::StatusCode;
use reqwest::{Client, ClientBuilder};

pub struct ProtectionBypassConfiguration {
    pub user_agent: String,
}

/// Try to get a client which passes the Cloudflare bot check Crunchyroll has installed.
/// If returning [`None`] no matching client could be built, if the second tuple item is [`None`]
/// the default client builder passed as argument
/// (or [`CrunchyrollBuilder::predefined_client_builder`] which is used if the `client_builder`
/// argument is [`None`]) was able to bypass the bot check with the default configurations.
pub async fn get_bypass_client<S, F>(
    user_agents: Vec<S>,
    client_builder: Option<F>,
) -> Result<Option<(Client, Option<ProtectionBypassConfiguration>)>>
where
    S: AsRef<str>,
    F: Fn() -> ClientBuilder,
{
    let client_builder: Box<dyn Fn() -> ClientBuilder> = if let Some(cb) = client_builder {
        Box::new(cb)
    } else {
        Box::new(CrunchyrollBuilder::predefined_client_builder)
    };

    // using this url instead of 'https://www.crunchyroll.com' as the bot protection
    // seems to be less strict on the root page
    let check_url = "https://www.crunchyroll.com/auth/v1/token";

    let mut client = client_builder().build().unwrap();
    if client.post(check_url).send().await?.status() != StatusCode::FORBIDDEN {
        return Ok(Some((client, None)));
    }

    for user_agent in user_agents {
        client = client_builder()
            .user_agent(user_agent.as_ref())
            .build()
            .unwrap();
        if client.post(check_url).send().await?.status() != StatusCode::FORBIDDEN {
            return Ok(Some((
                client,
                Some(ProtectionBypassConfiguration {
                    user_agent: user_agent.as_ref().to_string(),
                }),
            )));
        }
    }

    Ok(None)
}
