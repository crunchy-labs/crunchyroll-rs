use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use serde::de::DeserializeOwned;
use serde::Deserialize;

pub(crate) type Result<T, E = CrunchyrollError> = core::result::Result<T, E>;

#[derive(Debug)]
pub enum CrunchyrollError {
    RequestError(CrunchyrollErrorContext),
    DecodeError(CrunchyrollErrorContext),

    LoginError(CrunchyrollErrorContext)
}

impl Display for CrunchyrollError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CrunchyrollError::RequestError(context) => write!(f, "{}", context.message),
            CrunchyrollError::DecodeError(context) => write!(f, "{}", context.message),
            CrunchyrollError::LoginError(context) => write!(f, "{}", context.message)
        }
    }
}

impl Error for CrunchyrollError {}

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

    if let Ok(err) = serde_json::from_value::<CodeContextError>(value) {
        let mut details: Vec<String> = vec![];

        for item in err.context.iter() {
            details.push(format!("{}: {}", item.field, item.code))
        }

        return Err(CrunchyrollError::RequestError(
            CrunchyrollErrorContext{ message: format!("{} ({}) - {}", err.error, err.code, details.join(", ")) }
        ));
    }
    Ok(())
}

pub(crate) fn check_request_error<T: DeserializeOwned>(value: serde_json::Value) -> Result<T> {
    is_request_error(value.to_owned())?;
    serde_json::from_value::<T>(value).map_err(|e| CrunchyrollError::DecodeError(
        CrunchyrollErrorContext{ message: e.to_string() }
    ))
}
