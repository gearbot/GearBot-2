use std::fmt::{Display, Formatter};

use sqlx::Error;

use super::guild::CURRENT_CONFIG_VERSION;

pub enum DatastoreError {
    Sqlx(sqlx::Error),
    Serde(serde_json::Error),
    UnsupportedConfigVersion(i32),
}

impl Display for DatastoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DatastoreError::Sqlx(e) => write!(f, "Database error occurred: {:?}", e),
            DatastoreError::Serde(e) => write!(f, "Serde error: {}", e),
            DatastoreError::UnsupportedConfigVersion(v) => write!(
                f,
                "Config is of version {} but this application only supports up to {} at this time",
                v, CURRENT_CONFIG_VERSION
            ),
        }
    }
}

impl From<sqlx::Error> for DatastoreError {
    fn from(e: Error) -> Self {
        DatastoreError::Sqlx(e)
    }
}

impl From<serde_json::Error> for DatastoreError {
    fn from(e: serde_json::Error) -> Self {
        DatastoreError::Serde(e)
    }
}
