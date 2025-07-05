use std::fmt;

/// Custom error type for the search engine
#[derive(Debug)]
pub enum SearchEngineError {
    /// Tantivy-related errors
    TantivyError(tantivy::TantivyError),

    /// I/O errors
    IoError(std::io::Error),

    /// Serialization/deserialization errors
    SerdeError(serde_json::Error),

    /// Schema-related errors
    SchemaError(String),

    /// Collection-related errors
    CollectionError(String),

    /// Query parsing errors
    QueryError(String),

    /// Index errors
    IndexError(String),

    /// Configuration errors
    ConfigError(String),

    /// Search errors
    SearchError(String),

    /// Generic error with custom message
    CustomError(String),
}

impl fmt::Display for SearchEngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchEngineError::TantivyError(e) => write!(f, "Tantivy error: {}", e),
            SearchEngineError::IoError(e) => write!(f, "I/O error: {}", e),
            SearchEngineError::SerdeError(e) => write!(f, "Serialization error: {}", e),
            SearchEngineError::SchemaError(msg) => write!(f, "Schema error: {}", msg),
            SearchEngineError::CollectionError(msg) => write!(f, "Collection error: {}", msg),
            SearchEngineError::QueryError(msg) => write!(f, "Query error: {}", msg),
            SearchEngineError::IndexError(msg) => write!(f, "Index error: {}", msg),
            SearchEngineError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            SearchEngineError::SearchError(msg) => write!(f, "Search error: {}", msg),
            SearchEngineError::CustomError(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for SearchEngineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SearchEngineError::TantivyError(e) => Some(e),
            SearchEngineError::IoError(e) => Some(e),
            SearchEngineError::SerdeError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<tantivy::TantivyError> for SearchEngineError {
    fn from(error: tantivy::TantivyError) -> Self {
        SearchEngineError::TantivyError(error)
    }
}

impl From<std::io::Error> for SearchEngineError {
    fn from(error: std::io::Error) -> Self {
        SearchEngineError::IoError(error)
    }
}

impl From<serde_json::Error> for SearchEngineError {
    fn from(error: serde_json::Error) -> Self {
        SearchEngineError::SerdeError(error)
    }
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, SearchEngineError>;
