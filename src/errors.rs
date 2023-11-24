use core::fmt;
use objects::AccountError;

// CLIENT ERROR
// ================================================================================================

#[derive(Debug)]
pub enum ClientError {
    StoreError(StoreError),
    AccountError(AccountError),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::StoreError(err) => write!(f, "store error: {err}"),
            ClientError::AccountError(err) => write!(f, "account error: {err}"),
        }
    }
}

impl From<StoreError> for ClientError {
    fn from(err: StoreError) -> Self {
        Self::StoreError(err)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ClientError {}

// STORE ERROR
// ================================================================================================

#[derive(Debug)]
pub enum StoreError {
    ConnectionError(rusqlite::Error),
    MigrationError(rusqlite_migration::Error),
    QueryError(rusqlite::Error),
    InputSerializationError(serde_json::Error),
    DataDeserializationError(serde_json::Error),
    NotFound,
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use StoreError::*;
        match self {
            ConnectionError(err) => write!(f, "failed to connect to the database: {err}"),
            MigrationError(err) => write!(f, "failed to update the database: {err}"),
            QueryError(err) => write!(f, "failed to retrieve data from the database: {err}"),
            InputSerializationError(err) => {
                write!(f, "error trying to serialize inputs for the store: {err}")
            }
            DataDeserializationError(err) => {
                write!(f, "error deserializing data from the store: {err}")
            }
            NotFound => write!(f, "requested data was not found in the store"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for StoreError {}
