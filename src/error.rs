use reqwest::Response;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub(crate) type Result<T, E = CrunchyrollError> = core::result::Result<T, E>;

/// Crate specfic error types.
#[derive(Clone, Debug)]
pub enum CrunchyrollError {
    /// Error was caused by something library internal. This only happens if something was
    /// implemented incorrectly (which hopefully should never be the case) or if Crunchyroll
    /// surprisingly changed specific parts of their api which broke a part of this crate.
    Internal(CrunchyrollErrorContext),

    /// Some sort of error occurred while requesting the Crunchyroll api.
    Request(CrunchyrollErrorContext),
    /// While decoding the api response body something went wrong.
    Decode(CrunchyrollErrorContext),

    /// Something went wrong while logging in.
    Authentication(CrunchyrollErrorContext),

    /// Generally malformed or invalid user input.
    Input(CrunchyrollErrorContext),
}

impl Display for CrunchyrollError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CrunchyrollError::Internal(context) => write!(f, "{}", context),
            CrunchyrollError::Request(context) => write!(f, "{}", context),
            CrunchyrollError::Decode(context) => write!(f, "{}", context),
            CrunchyrollError::Authentication(context) => write!(f, "{}", context),
            CrunchyrollError::Input(context) => write!(f, "{}", context),
        }
    }
}

impl Error for CrunchyrollError {}

impl From<serde_json::Error> for CrunchyrollError {
    fn from(err: serde_json::Error) -> Self {
        Self::Decode(CrunchyrollErrorContext::new(err.to_string()))
    }
}

impl From<reqwest::Error> for CrunchyrollError {
    fn from(err: reqwest::Error) -> Self {
        let mut context = CrunchyrollErrorContext::new(err.to_string());
        if let Some(url) = err.url() {
            context = context.with_url(url.clone());
        }

        if err.is_request()
            || err.is_redirect()
            || err.is_timeout()
            || err.is_connect()
            || err.is_body()
            || err.is_status()
        {
            CrunchyrollError::Request(context)
        } else if err.is_decode() {
            CrunchyrollError::Decode(context)
        } else if err.is_builder() {
            CrunchyrollError::Internal(context)
        } else {
            CrunchyrollError::Internal(CrunchyrollErrorContext::new(format!(
                "Could not determine request error type - {}",
                err
            )))
        }
    }
}

impl From<std::io::Error> for CrunchyrollError {
    fn from(err: std::io::Error) -> Self {
        CrunchyrollError::Request(CrunchyrollErrorContext::new(err.to_string()))
    }
}

/// Information about a [`CrunchyrollError`].
#[derive(Clone, Debug)]
pub struct CrunchyrollErrorContext {
    pub message: String,
    pub url: Option<String>,
    pub value: Option<String>,
}

impl Display for CrunchyrollErrorContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut res = self.message.clone();

        if let Some(url) = &self.url {
            res.push_str(&format!(" ({})", url));
        }
        if let Some(value) = &self.value {
            res.push_str(&format!(": {}", value));
        }

        write!(f, "{}", res)
    }
}

impl From<String> for CrunchyrollErrorContext {
    fn from(string: String) -> Self {
        CrunchyrollErrorContext::new(string)
    }
}

impl From<&str> for CrunchyrollErrorContext {
    fn from(str: &str) -> Self {
        CrunchyrollErrorContext::new(str)
    }
}

impl CrunchyrollErrorContext {
    pub(crate) fn new<S: ToString>(message: S) -> Self {
        Self {
            message: message.to_string(),
            url: None,
            value: None,
        }
    }

    pub(crate) fn with_url<S: AsRef<str>>(mut self, url: S) -> Self {
        self.url = Some(url.as_ref().to_string());

        self
    }

    pub(crate) fn with_value(mut self, value: &[u8]) -> Self {
        self.value = Some(format!(
            ": {}",
            std::str::from_utf8(value).unwrap_or("-- not displayable --")
        ));

        self
    }
}

pub(crate) fn is_request_error(value: Value) -> Result<()> {
    #[derive(Debug, Deserialize)]
    struct CodeFieldContext {
        code: String,
        field: String,
    }

    #[derive(Debug, Deserialize)]
    struct MessageType {
        message: String,
        #[serde(rename = "type")]
        error_type: String,
    }
    #[derive(Debug, Deserialize)]
    struct CodeContextError {
        code: String,
        context: Vec<CodeFieldContext>,
        #[serde(alias = "error")]
        message: Option<String>,
    }
    #[derive(Debug, Deserialize)]
    struct ConstraintsErrorContext {
        code: String,
        violated_constraints: Vec<(String, String)>,
    }
    #[derive(Debug, Deserialize)]
    struct ConstraintsError {
        code: String,
        context: Vec<ConstraintsErrorContext>,
    }

    if let Ok(err) = serde_json::from_value::<MessageType>(value.clone()) {
        return Err(CrunchyrollError::Request(
            format!("{} - {}", err.error_type, err.message).into(),
        ));
    } else if let Ok(err) = serde_json::from_value::<CodeContextError>(value.clone()) {
        let mut details: Vec<String> = vec![];

        for item in err.context.iter() {
            details.push(format!("{}: {}", item.field, item.code))
        }

        return if let Some(message) = err.message {
            Err(CrunchyrollError::Request(
                format!("{} ({}) - {}", message, err.code, details.join(", ")).into(),
            ))
        } else {
            Err(CrunchyrollError::Request(
                format!("({}) - {}", err.code, details.join(", ")).into(),
            ))
        };
    } else if let Ok(err) = serde_json::from_value::<ConstraintsError>(value) {
        let details = err
            .context
            .iter()
            .map(|e| {
                format!(
                    "{}: ({})",
                    e.code,
                    e.violated_constraints
                        .iter()
                        .map(|(key, value)| format!("{}: {}", key, value))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            })
            .collect::<Vec<String>>();

        return Err(CrunchyrollError::Request(
            format!("{}: {}", err.code, details.join(", ")).into(),
        ));
    }
    Ok(())
}

pub(crate) async fn check_request<T: DeserializeOwned>(url: String, resp: Response) -> Result<T> {
    if resp.status().as_u16() == 429 {
        let retry_secs = if let Some(retry_after) = resp.headers().get(http::header::RETRY_AFTER) {
            retry_after.to_str().map_or(None, |retry_after_secs| {
                retry_after_secs.parse::<u32>().ok()
            })
        } else {
            None
        };

        return Err(CrunchyrollError::Request(
            CrunchyrollErrorContext::new(format!(
                "Rate limit detected. {}",
                retry_secs.map_or("Try again later".to_string(), |secs| format!(
                    "Try again in {} seconds",
                    secs
                ))
            ))
            .with_url(resp.url()),
        ));
    } else if resp.status().is_client_error() {
        return Err(CrunchyrollError::Request(
            CrunchyrollErrorContext::new("Unexpected client error").with_url(resp.url()),
        ));
    } else if resp.status().is_server_error() {
        return Err(CrunchyrollError::Request(
            CrunchyrollErrorContext::new("Server error detected. Try again later")
                .with_url(resp.url()),
        ));
    }

    let content_length = resp.content_length().unwrap_or(0);
    let _raw = resp.bytes().await.unwrap();
    let mut raw: &[u8] = _raw.as_ref();

    // to ensure compatibility with `T`, convert a empty response to {}
    if raw.is_empty() && (content_length == 0) {
        raw = "{}".as_bytes();
    }

    let value: Value = serde_json::from_slice(raw).map_err(|e| {
        CrunchyrollError::Decode(
            CrunchyrollErrorContext::new(format!("{} at {}:{}", e, e.line(), e.column()))
                .with_url(&url)
                .with_value(raw),
        )
    })?;
    is_request_error(value.clone()).map_err(|e| {
        if let CrunchyrollError::Request(context) = e {
            CrunchyrollError::Request(context.with_url(&url))
        } else {
            e
        }
    })?;
    serde_json::from_value::<T>(value).map_err(|e| {
        CrunchyrollError::Decode(
            CrunchyrollErrorContext::new(format!("{} at {}:{}", e, e.line(), e.column()))
                .with_url(&url)
                .with_value(raw),
        )
    })
}
