use serde_json::Value;

#[derive(Debug)]
pub enum Error {
    SerdeJsonError(serde_json::Error),
    /// InvalidJsonKey(Key, Value)
    InvalidJsonKey(String, Value),
    /// InvalidJsonType(Expected, Actual_Value)
    InvalidJsonType(String, Value),
    ReqwestError(reqwest::Error),
    /// MissingHeader(Missing_Header, All_Headers)
    MissingHeader(String, String),
    InvalidHeader(String, String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{self:#?}")
    }
}

impl From<serde_json::Error> for Error {
    fn from(o: serde_json::Error) -> Self {
        Self::SerdeJsonError(o)
    }
}

impl From<reqwest::Error> for Error {
    fn from(o: reqwest::Error) -> Self {
        Self::ReqwestError(o)
    }
}
