use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use serde::de::DeserializeOwned;
use serde::Deserialize;

pub(crate) type Result<T, E = CrunchyrollError> = core::result::Result<T, E>;

#[derive(Debug)]
pub enum CrunchyrollError {
    Internal(CrunchyrollErrorContext),
    Request(CrunchyrollErrorContext),
    Decode(CrunchyrollErrorContext),

    Authentication(CrunchyrollErrorContext)
}

impl Display for CrunchyrollError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CrunchyrollError::Internal(context) => write!(f, "{}", context.message),
            CrunchyrollError::Request(context) => write!(f, "{}", context.message),
            CrunchyrollError::Decode(context) => write!(f, "{}", context.message),
            CrunchyrollError::Authentication(context) => write!(f, "{}", context.message)
        }
    }
}

impl Error for CrunchyrollError {}

impl From<serde_json::Error> for CrunchyrollError {
    fn from(err: serde_json::Error) -> Self {
        Self::Decode(
            CrunchyrollErrorContext{ message: err.to_string() }
        )
    }
}

impl From<serde_urlencoded::de::Error> for CrunchyrollError {
    fn from(err: serde_urlencoded::de::Error) -> Self {
        Self::Decode(
            CrunchyrollErrorContext{ message: err.to_string() }
        )
    }
}

impl From<serde_urlencoded::ser::Error> for CrunchyrollError {
    fn from(err: serde_urlencoded::ser::Error) -> Self {
        Self::Decode(
            CrunchyrollErrorContext{ message: err.to_string() }
        )
    }
}

#[derive(Debug)]
pub struct CrunchyrollErrorContext {
    pub message: String
}

pub(crate) fn is_request_error(value: serde_json::Value) -> Result<()> {
    #[derive(Debug, Deserialize)]
    struct CodeFieldContext {
        code: String,
        field: String,
    }

    #[derive(Debug, Deserialize)]
    struct CodeContextError {
        code: String,
        context: Vec<CodeFieldContext>,
        error: String
    }
    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct CodeContextError2 {
        code: String,
        // I haven't encountered a error with a populated value for this yet
        context: serde_json::Value,
        message: String
    }

    if let Ok(err) = serde_json::from_value::<CodeContextError>(value.clone()) {
        let mut details: Vec<String> = vec![];

        for item in err.context.iter() {
            details.push(format!("{}: {}", item.field, item.code))
        }

        return Err(CrunchyrollError::Request(
            CrunchyrollErrorContext{ message: format!("{} ({}) - {}", err.error, err.code, details.join(", ")) }
        ));
    } else if let Ok(err) = serde_json::from_value::<CodeContextError2>(value) {
        return Err(CrunchyrollError::Request(
            CrunchyrollErrorContext{ message: format!("{} ({})", err.message, err.code) }
        ))
    }
    Ok(())
}

pub(crate) fn check_request_error<T: DeserializeOwned>(value: serde_json::Value) -> Result<T> {
    is_request_error(value.to_owned())?;
    serde_json::from_value::<T>(value).map_err(|e| CrunchyrollError::Decode(
        CrunchyrollErrorContext{ message: e.to_string() }
    ))
}
