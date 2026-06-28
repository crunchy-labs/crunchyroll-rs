//! Library specific errors.

use reqwest::{Response, StatusCode, Url};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use std::error::Error as StdError;
use std::fmt::{Debug, Display, Formatter};

pub(crate) type Result<T, E = Error> = core::result::Result<T, E>;

/// Errors that may occur.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    source: Option<Box<dyn StdError + Send + Sync>>,
    url: Option<String>,

    message: Option<String>,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg_fn = |default: &'static str| {
            self.message.as_ref().cloned().unwrap_or_else(|| {
                self.source
                    .as_ref()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| default.to_string())
            })
        };

        let (message, suffix) = match &self.kind {
            ErrorKind::Internal => (msg_fn("internal error"), None),
            ErrorKind::Request { status } => {
                let message = msg_fn("request error");
                if let Some(status) = &status {
                    (message, Some(format!(" (http {})", status.as_u16())))
                } else {
                    (message, None)
                }
            }
            ErrorKind::Decode { content } => {
                let message = msg_fn("decoding error");
                if let Some(content) = content {
                    (
                        message,
                        Some(format!(": {}", String::from_utf8_lossy(content.as_slice()))),
                    )
                } else {
                    (message, None)
                }
            }
            ErrorKind::Authentication => (msg_fn("authentication error"), None),
            ErrorKind::Input => (msg_fn("input error"), None),
            ErrorKind::Block { body } => {
                let message = msg_fn("cloudflare blocked");
                (message, Some(format!(": {}", body)))
            }
        };

        if let Some(url) = &self.url {
            write!(f, "{} at {}{}", message, url, suffix.unwrap_or_default())
        } else {
            write!(f, "{}{}", message, suffix.unwrap_or_default())
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_ref().map(|e| &**e as _)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self {
            kind: ErrorKind::Decode { content: None },
            source: Some(err.into()),
            url: None,
            message: None,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        let (kind, message) = if err.is_request()
            || err.is_redirect()
            || err.is_timeout()
            || err.is_connect()
            || err.is_body()
            || err.is_status()
        {
            (
                ErrorKind::Request {
                    status: err.status(),
                },
                None,
            )
        } else if err.is_decode() {
            (ErrorKind::Decode { content: None }, None)
        } else if err.is_builder() {
            (ErrorKind::Internal, None)
        } else {
            (
                ErrorKind::Internal,
                Some("Could not determine request error type - {err}".to_string()),
            )
        };

        let url = err.url().map(Url::to_string);
        Self {
            kind,
            source: Some(err.into()),
            url,
            message,
        }
    }
}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    pub(crate) fn error_from_kind<S: AsRef<str>>(kind: ErrorKind, msg: S) -> Self {
        Self {
            kind,
            source: None,
            url: None,
            message: Some(msg.as_ref().to_string()),
        }
    }

    pub(crate) fn error_from_other_error_and_url<
        E: Into<Box<dyn StdError + Send + Sync>>,
        U: AsRef<str>,
    >(
        other_error: E,
        kind: ErrorKind,
        url: U,
    ) -> Self {
        Self {
            kind,
            source: Some(other_error.into()),
            url: Some(url.as_ref().to_string()),
            message: None,
        }
    }

    pub(crate) fn error_from_kind_and_url<S: AsRef<str>, U: AsRef<str>>(
        kind: ErrorKind,
        url: U,
        msg: S,
    ) -> Self {
        Self {
            kind,
            source: None,
            url: Some(url.as_ref().to_string()),
            message: Some(msg.as_ref().to_string()),
        }
    }
}

/// Specific error types.
#[derive(Debug)]
pub enum ErrorKind {
    /// Error was caused by something library internal. This only happens if something was
    /// implemented incorrectly (which hopefully should never be the case) or if Crunchyroll
    /// surprisingly changed specific parts of their api which broke a part of this crate.
    Internal,
    /// Some sort of error occurred while requesting the Crunchyroll api.
    Request { status: Option<StatusCode> },
    /// While decoding the api response body something went wrong.
    Decode { content: Option<Vec<u8>> },
    /// Something went wrong while logging in.
    Authentication,
    /// Malformed or invalid user input.
    Input,
    /// When a request got blocked. Currently, this only triggers when the cloudflare bot
    /// protection is detected.
    Block {
        /// HTML/text body of the block response.
        body: String,
    },
}

pub(crate) fn is_request_error<U: AsRef<str>>(
    value: Value,
    url: U,
    status: &StatusCode,
) -> Result<()> {
    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    #[allow(clippy::enum_variant_names)]
    enum ErrorType {
        MessageTypeError {
            message: String,
            r#type: String,
        },
        CodeError {
            code: String,
            context: Vec<CodeErrorContext>,
            #[serde(alias = "error")]
            message: Option<String>,
        },
        GenericError {
            error: Value,
            #[serde(flatten)]
            other: Map<String, Value>,
        },
    }

    #[derive(Debug, Deserialize)]
    struct CodeErrorContext {
        code: String,
        #[serde(flatten)]
        other: Map<String, Value>,
    }

    let Ok(error_type) = serde_json::from_value::<ErrorType>(value) else {
        return Ok(());
    };
    let error_msg = match error_type {
        ErrorType::MessageTypeError { message, r#type } => {
            format!("{type} - {message}")
        }
        ErrorType::CodeError {
            code,
            context,
            message,
        } => {
            let mut msg = if let Some(message) = message {
                format!("{message} - {code}")
            } else {
                code
            };
            if !context.is_empty() {
                let details: Vec<String> = context
                    .into_iter()
                    .map(|c| format!("{}: {}", c.code, serde_json::to_string(&c.other).unwrap()))
                    .collect();
                msg += &format!(": ({})", details.join(", "))
            }
            msg
        }
        ErrorType::GenericError { error, other } => {
            let mut msg = match error {
                Value::Number(num) => format!("error {num}"),
                _ => error.to_string(),
            };
            if !other.is_empty() {
                msg += &format!(": {}", serde_json::to_string(&other).unwrap())
            }
            msg
        }
    };

    Err(Error {
        kind: ErrorKind::Request {
            status: Some(*status),
        },
        source: None,
        url: Some(url.as_ref().to_string()),
        message: Some(error_msg),
    })
}

pub(crate) async fn check_request<T: DeserializeOwned>(resp: Response) -> Result<T> {
    let url = resp.url().clone();
    let content_length = resp.content_length().unwrap_or(0);
    let status = resp.status();
    let _raw = match resp.status().as_u16() {
        403 => {
            let raw = resp.bytes().await?;
            if raw.starts_with(b"<!DOCTYPE html>")
                && raw
                    .windows(31)
                    .any(|w| w == b"<title>Just a moment...</title>")
            {
                return Err(Error {
                    kind: ErrorKind::Block {
                        body: String::from_utf8_lossy(raw.as_ref()).to_string(),
                    },
                    source: None,
                    url: Some(url.to_string()),
                    message: Some("Triggered Cloudflare bot protection".to_string()),
                });
            }
            raw
        }
        404 => {
            return Err(Error {
                kind: ErrorKind::Request {
                    status: Some(resp.status()),
                },
                source: None,
                url: Some(url.to_string()),
                message: Some("The requested resource is not present".to_string()),
            });
        }
        429 => {
            let retry_secs =
                if let Some(retry_after) = resp.headers().get(reqwest::header::RETRY_AFTER) {
                    retry_after.to_str().map_or(None, |retry_after_secs| {
                        retry_after_secs.parse::<u32>().ok()
                    })
                } else {
                    None
                };

            return Err(Error {
                kind: ErrorKind::Request {
                    status: Some(resp.status()),
                },
                source: None,
                url: Some(url.to_string()),
                message: Some(format!(
                    "Rate limit detected. {}",
                    retry_secs.map_or("Try again later".to_string(), |secs| format!(
                        "Try again in {secs} seconds"
                    ))
                )),
            });
        }
        _ => resp.bytes().await?,
    };
    let mut raw: &[u8] = _raw.as_ref();

    // to ensure compatibility with `T`, convert a empty response to {}
    if raw.is_empty() && (content_length == 0) {
        raw = "{}".as_bytes();
    }

    let value: Value = serde_json::from_slice(raw).map_err(|e| Error {
        kind: ErrorKind::Decode {
            content: Some(raw.to_vec()),
        },
        source: Some(e.into()),
        url: Some(url.to_string()),
        message: None,
    })?;
    is_request_error(value.clone(), &url, &status)?;
    serde_json::from_value::<T>(value).map_err(|e| Error {
        kind: ErrorKind::Decode {
            content: Some(raw.to_vec()),
        },
        source: Some(e.into()),
        url: Some(url.to_string()),
        message: None,
    })
}
