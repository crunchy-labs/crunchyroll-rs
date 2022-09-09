use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub(crate) type Result<T, E = CrunchyrollError> = core::result::Result<T, E>;

#[derive(Debug)]
pub enum CrunchyrollError {
    Internal(CrunchyrollErrorContext),
    External(CrunchyrollErrorContext),

    Request(CrunchyrollErrorContext),
    Decode(CrunchyrollErrorContext),

    Authentication(CrunchyrollErrorContext),
}

impl Display for CrunchyrollError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CrunchyrollError::Internal(context) => write!(f, "{}", context),
            CrunchyrollError::External(context) => write!(f, "{}", context),
            CrunchyrollError::Request(context) => write!(f, "{}", context),
            CrunchyrollError::Decode(context) => write!(f, "{}", context),
            CrunchyrollError::Authentication(context) => write!(f, "{}", context),
        }
    }
}

impl Error for CrunchyrollError {}

impl From<serde_json::Error> for CrunchyrollError {
    fn from(err: serde_json::Error) -> Self {
        Self::Decode(CrunchyrollErrorContext::new(err.to_string()))
    }
}

impl From<serde_urlencoded::de::Error> for CrunchyrollError {
    fn from(err: serde_urlencoded::de::Error) -> Self {
        Self::Decode(CrunchyrollErrorContext::new(err.to_string()))
    }
}

impl From<serde_urlencoded::ser::Error> for CrunchyrollError {
    fn from(err: serde_urlencoded::ser::Error) -> Self {
        Self::Decode(CrunchyrollErrorContext::new(err.to_string()))
    }
}

impl From<reqwest::Error> for CrunchyrollError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_request()
            || err.is_redirect()
            || err.is_timeout()
            || err.is_connect()
            || err.is_body()
            || err.is_status()
        {
            CrunchyrollError::Request(CrunchyrollErrorContext::new(err.to_string()))
        } else if err.is_decode() {
            CrunchyrollError::Decode(CrunchyrollErrorContext::new(err.to_string()))
        } else if err.is_builder() {
            CrunchyrollError::Internal(CrunchyrollErrorContext::new(err.to_string()))
        } else {
            CrunchyrollError::Internal(CrunchyrollErrorContext::new(format!(
                "Could not determine request error type - {}",
                err
            )))
        }
    }
}

#[derive(Debug)]
pub struct CrunchyrollErrorContext {
    pub message: String,
    pub url: Option<String>,
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

impl CrunchyrollErrorContext {
    pub(crate) fn new(message: String) -> Self {
        Self {
            message,
            url: None,
            value: None,
        }
    }

    pub(crate) fn with_url(mut self, url: String) -> Self {
        self.url = Some(url);

        self
    }

    pub(crate) fn with_value(mut self, value: &[u8]) -> Self {
        self.value = Some(value.to_vec());

        self
    }
}

pub(crate) fn is_request_error(value: serde_json::Value) -> Result<()> {
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
        error: String,
    }
    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct CodeContextError2 {
        code: String,
        // I haven't encountered a error with a populated value for this yet
        context: serde_json::Value,
        message: String,
    }

    if let Ok(err) = serde_json::from_value::<MessageType>(value.clone()) {
        return Err(CrunchyrollError::Request(CrunchyrollErrorContext::new(
            format!("{} - {}", err.error_type, err.message),
        )));
    } else if let Ok(err) = serde_json::from_value::<CodeContextError>(value.clone()) {
        let mut details: Vec<String> = vec![];

        for item in err.context.iter() {
            details.push(format!("{}: {}", item.field, item.code))
        }

        return Err(CrunchyrollError::Request(CrunchyrollErrorContext::new(
            format!("{} ({}) - {}", err.error, err.code, details.join(", ")),
        )));
    } else if let Ok(err) = serde_json::from_value::<CodeContextError2>(value) {
        return Err(CrunchyrollError::Request(CrunchyrollErrorContext::new(
            format!("{} ({})", err.message, err.code),
        )));
    }
    Ok(())
}

pub(crate) async fn check_request<T: DeserializeOwned>(resp: reqwest::Response) -> Result<T> {
    let url = resp.url().to_string();
    let raw = resp.bytes().await?;

    let value: serde_json::Value = serde_json::from_slice(raw.as_ref()).map_err(|e| {
        CrunchyrollError::Decode(
            CrunchyrollErrorContext::new(format!("{} at {}:{}", e, e.line(), e.column()))
                .with_url(url.clone())
                .with_value(raw.as_ref()),
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
                .with_value(raw.as_ref()),
        )
    })
}
