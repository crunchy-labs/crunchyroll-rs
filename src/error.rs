use reqwest::{IntoUrl, Url};
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

/// Information about a [`CrunchyrollError`].
#[derive(Clone, Debug)]
pub struct CrunchyrollErrorContext {
    pub message: String,
    pub url: Option<Url>,
    pub value: Option<Vec<u8>>,
}

impl Display for CrunchyrollErrorContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut res = self.message.clone();

        if let Some(url) = &self.url {
            res.push_str(format!(" ({})", url).as_str());
        }
        if let Some(value) = &self.value {
            res.push_str(
                format!(
                    ": {}",
                    std::str::from_utf8(value.as_slice()).unwrap_or("-- not displayable --")
                )
                .as_str(),
            );
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

    pub(crate) fn with_url<U: IntoUrl>(mut self, url: U) -> Self {
        self.url = Some(url.into_url().unwrap());

        self
    }

    pub(crate) fn with_value(mut self, value: &[u8]) -> Self {
        self.value = Some(value.to_vec());

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

    if let Ok(err) = serde_json::from_value::<MessageType>(value.clone()) {
        return Err(CrunchyrollError::Request(
            format!("{} - {}", err.error_type, err.message).into(),
        ));
    } else if let Ok(err) = serde_json::from_value::<CodeContextError>(value) {
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
    }
    Ok(())
}

pub(crate) async fn check_request<T: DeserializeOwned>(resp: reqwest::Response) -> Result<T> {
    let content_length = resp.content_length();
    let url = resp.url().to_string();
    let _raw = resp.bytes().await?;
    let mut raw = _raw.as_ref();

    // to ensure compatibility with `T`, convert a empty response to {}
    if raw.is_empty() && content_length.unwrap_or(1) == 0 {
        raw = "{}".as_bytes();
    }

    let value: Value = serde_json::from_slice(raw).map_err(|e| {
        CrunchyrollError::Decode(
            CrunchyrollErrorContext::new(format!("{} at {}:{}", e, e.line(), e.column()))
                .with_url(url.clone())
                .with_value(raw),
        )
    })?;
    is_request_error(value.clone()).map_err(|e| {
        if let CrunchyrollError::Request(context) = e {
            CrunchyrollError::Request(context.with_url(url.clone()))
        } else {
            e
        }
    })?;
    serde_json::from_value::<T>(value).map_err(|e| {
        CrunchyrollError::Decode(
            CrunchyrollErrorContext::new(format!("{} at {}:{}", e, e.line(), e.column()))
                .with_url(url)
                .with_value(raw),
        )
    })
}
