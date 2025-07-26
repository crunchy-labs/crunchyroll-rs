//! Library specific errors.

use reqwest::{Response, StatusCode};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use std::fmt::{Debug, Display, Formatter};

pub(crate) type Result<T, E = Error> = core::result::Result<T, E>;

/// Crate specific error types.
#[derive(Clone, Debug)]
pub enum Error {
    /// Error was caused by something library internal. This only happens if something was
    /// implemented incorrectly (which hopefully should never be the case) or if Crunchyroll
    /// surprisingly changed specific parts of their api which broke a part of this crate.
    Internal { message: String },

    /// Some sort of error occurred while requesting the Crunchyroll api.
    Request {
        message: String,
        status: Option<StatusCode>,
        /// The url which caused the error.
        url: String,
    },
    /// While decoding the api response body something went wrong.
    Decode {
        message: String,
        /// The content which failed to get decoded. Might be empty if the error got triggered by
        /// the [`From<serde_json::Error>`] implementation for this enum.
        content: Vec<u8>,
        /// The url which caused the error. Might be empty if the error got triggered by the
        /// [`From<serde_json::Error>`] implementation for this enum.
        url: String,
    },

    /// Something went wrong while logging in.
    Authentication { message: String },

    /// Generally malformed or invalid user input.
    Input { message: String },

    /// When the request got blocked. Currently this only triggers when the cloudflare bot
    /// protection is detected.
    Block {
        message: String,
        /// HTML/text body of the block response.
        body: String,
        /// The url which caused the error.
        url: String,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Internal { message } => write!(f, "{message}"),
            Error::Request { message, url, .. } => {
                // the url can be 'n/a' when the error got triggered by the [`From<reqwest::Error>`]
                // implementation for this error struct
                if url != "n/a" {
                    write!(f, "{message} ({url})")
                } else {
                    write!(f, "{message}")
                }
            }
            Error::Decode {
                message,
                content,
                url,
            } => {
                let mut msg = message.clone();
                // the url is 'n/a' when the error got triggered by the [`From<serde_json::Error>`]
                // implementation for this error struct or [`VariantSegment::decrypt`]
                if url != "n/a" {
                    msg.push_str(&format!(" ({url})"))
                }
                if content.is_empty() {
                    write!(f, "{msg}")
                } else {
                    write!(f, "{}: {}", msg, String::from_utf8_lossy(content.as_ref()))
                }
            }
            Error::Authentication { message } => write!(f, "{message}"),
            Error::Input { message } => write!(f, "{message}"),
            Error::Block { message, body, url } => write!(f, "{message} ({url}): {body}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Decode {
            message: err.to_string(),
            content: vec![],
            url: "n/a".to_string(),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if err.is_request()
            || err.is_redirect()
            || err.is_timeout()
            || err.is_connect()
            || err.is_body()
            || err.is_status()
        {
            Error::Request {
                message: err.to_string(),
                status: err.status(),
                url: err.url().map_or("n/a".to_string(), |url| url.to_string()),
            }
        } else if err.is_decode() {
            Error::Decode {
                message: err.to_string(),
                content: vec![],
                url: err.url().map_or("n/a".to_string(), |url| url.to_string()),
            }
        } else if err.is_builder() {
            Error::Internal {
                message: err.to_string(),
            }
        } else {
            Error::Internal {
                message: "Could not determine request error type - {err}".to_string(),
            }
        }
    }
}

pub(crate) fn is_request_error(value: Value, url: &str, status: &StatusCode) -> Result<()> {
    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    #[allow(clippy::enum_variant_names)]
    enum ErrorTypes {
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
            error: String,
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

    let error_msg = match serde_json::from_value::<ErrorTypes>(value) {
        Ok(ErrorTypes::MessageTypeError { message, r#type }) => {
            format!("{type} - {message}")
        }
        Ok(ErrorTypes::CodeError {
            code,
            context,
            message,
        }) => {
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
                msg += &format!(" ({})", details.join(", "))
            }
            msg
        }
        Ok(ErrorTypes::GenericError { error, other }) => {
            let mut msg = error;
            if !other.is_empty() {
                msg += &format!(" ({})", serde_json::to_string(&other).unwrap())
            }
            msg
        }
        Err(_) => return Ok(()),
    };
    Err(Error::Request {
        message: error_msg,
        status: Some(*status),
        url: url.to_string(),
    })
}

pub(crate) async fn check_request<T: DeserializeOwned>(url: String, resp: Response) -> Result<T> {
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
                return Err(Error::Block {
                    message: "Triggered Cloudflare bot protection".to_string(),
                    body: String::from_utf8_lossy(raw.as_ref()).to_string(),
                    url,
                });
            }
            raw
        }
        404 => {
            return Err(Error::Request {
                message: "The requested resource is not present".to_string(),
                status: Some(resp.status()),
                url,
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

            return Err(Error::Request {
                message: format!(
                    "Rate limit detected. {}",
                    retry_secs.map_or("Try again later".to_string(), |secs| format!(
                        "Try again in {secs} seconds"
                    ))
                ),
                status: Some(resp.status()),
                url,
            });
        }
        _ => resp.bytes().await?,
    };
    let mut raw: &[u8] = _raw.as_ref();

    // to ensure compatibility with `T`, convert a empty response to {}
    if raw.is_empty() && (content_length == 0) {
        raw = "{}".as_bytes();
    }

    let value: Value = serde_json::from_slice(raw).map_err(|e| Error::Decode {
        message: format!("{} at {}:{}", e, e.line(), e.column()),
        content: raw.to_vec(),
        url: url.clone(),
    })?;
    is_request_error(value.clone(), &url, &status)?;
    serde_json::from_value::<T>(value).map_err(|e| Error::Decode {
        message: format!("{} at {}:{}", e, e.line(), e.column()),
        content: raw.to_vec(),
        url,
    })
}
