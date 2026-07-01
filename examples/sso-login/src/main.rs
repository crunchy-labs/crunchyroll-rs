use crunchyroll_rs::Crunchyroll;
use crunchyroll_rs::crunchyroll::{CrunchyrollBuilder, DeviceIdentifier};
use crunchyroll_rs::media::StreamPlatform;
use http::Request;
use reqwest::Url;
use std::borrow::Cow;
use std::env;
use tao::dpi::LogicalSize;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::platform::run_return::EventLoopExtRunReturn;
use tao::platform::unix::WindowExtUnix;
use tao::window::WindowBuilder;
use uuid::Uuid;
use wry::{WebViewBuilder, WebViewBuilderExtUnix, WebViewId};

#[rustfmt::skip] // for scripts that may fetch this
const ANDROID_PHONE_BASIC_AUTH: &str = "azNqZnZsN2txcTdtcHhzd3VlZWg6Y2RJQnRNZjd1ZE9Td0pJcXE3ZnhiNktqQl9DbG5Fb0U=";
#[rustfmt::skip] // for scripts that may fetch this
const ANDROID_PHONE_SSO_CLIENT_ID: &str = "k3jfvl7kqq7mpxswueeh";
#[rustfmt::skip] // for scripts that may fetch this
const ANDROID_PHONE_USER_AGENT: &str = "Crunchyroll/3.112.1 Android/11 okhttp/5.3.2";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    let sso_credentials = get_sso_login_credentials_via_webview(ANDROID_PHONE_SSO_CLIENT_ID)?;

    let device_identifier = DeviceIdentifier {
        device_type: "ANDROID".to_string(),
        ..Default::default()
    };

    let client = CrunchyrollBuilder::predefined_client_builder()
        .user_agent(ANDROID_PHONE_USER_AGENT)
        .build()?;

    let _crunchyroll = Crunchyroll::builder()
        .client(client)
        .platform(
            StreamPlatform::AndroidPhone,
            ANDROID_PHONE_BASIC_AUTH.to_string(),
        )
        .login_with_oauth_code(
            sso_credentials.code,
            sso_credentials.code_verifier,
            device_identifier,
        )
        .await?;

    println!("Successfully logged in with SSO");

    Ok(())
}

struct SSOLoginCredentials {
    pub code: String,
    pub code_verifier: String,
}

fn get_sso_login_credentials_via_webview(client_id: &str) -> Result<SSOLoginCredentials> {
    #[cfg(target_os = "linux")]
    {
        if env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_err() {
            unsafe { env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1") }
        }
        if env::var("WEBKIT_DISABLE_COMPOSITING_MODE").is_err() {
            unsafe { env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1") }
        }
    }

    let challenge = Uuid::new_v4();
    let anonymous_id = Uuid::new_v4();
    let oauth_url = oauth_url(client_id, &challenge.to_string(), &anonymous_id.to_string());

    enum UserEvent {
        OAuthSuccess(String),
        OAuthCancelled,
    }

    let mut event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let event_loop_proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title("Crunchyroll SSO Login")
        .with_inner_size(LogicalSize::new(520., 720.))
        .with_resizable(true)
        .with_visible(true)
        .build(&event_loop)?;

    // custom protocol handler for crunchyroll:// protocol
    let protocol_handler_event_loop_proxy = event_loop_proxy.clone();
    let protocol_handler = move |_id: WebViewId, request: Request<Vec<u8>>| {
        let uri = request.uri();
        let url = Url::parse(&uri.to_string()).unwrap();
        let code = url
            .query_pairs()
            .find_map(|(key, value)| key.eq("code").then_some(value))
            .unwrap();

        let _ =
            protocol_handler_event_loop_proxy.send_event(UserEvent::OAuthSuccess(code.to_string()));

        http::Response::builder()
            .status(200)
            .header(http::header::CONTENT_TYPE, "text/plain")
            .body(Cow::Borrowed(&[] as &[u8]))
            .unwrap()
    };

    let webview_builder = WebViewBuilder::new()
        .with_url(oauth_url)
        .with_custom_protocol("sso.crunchyroll".to_string(), protocol_handler)
        .with_background_color((22, 23, 29, 255));

    #[cfg(not(unix))]
    let _webview = webview_builder.build(&window)?;
    #[cfg(unix)]
    let _webview = {
        let vbox = window.default_vbox().unwrap();
        webview_builder.build_gtk(vbox)?
    };

    let mut result = Err("SSO login cancelled".into());
    let event_loop_result = &mut result;
    let _ = event_loop.run_return(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        #[cfg(unix)]
        while gtk::events_pending() {
            gtk::main_iteration_do(false);
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::Focused(true),
                ..
            } => window.set_visible(true),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                let _ = event_loop_proxy.send_event(UserEvent::OAuthCancelled);
            }

            Event::UserEvent(UserEvent::OAuthSuccess(code)) => {
                *event_loop_result = Ok(SSOLoginCredentials {
                    code,
                    code_verifier: challenge.to_string(),
                });
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(UserEvent::OAuthCancelled) => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });

    result
}

fn oauth_url(client_id: &str, challenge: &str, anonymous_id: &str) -> String {
    format!(
        "https://sso.crunchyroll.com/authorize\
         ?client_id={client_id}\
         &redirect_uri=sso.crunchyroll://auth\
         &scope=offline_access\
         &code_challenge={challenge}\
         &code_challenge_method=plain\
         &response_type=code\
         &anonymous_id={anonymous_id}"
    )
}
