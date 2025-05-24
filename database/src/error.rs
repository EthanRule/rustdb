use std::fmt;
use std::io;
use serde_json;

#[derive(Debug)]
pub enum DatabaseError {
    Storage(String),
    Document(String),
    Query(String),
    Index(String),
    Network(String),
    Validation(String),
    Io(io::Error),
    Json(serde_json::Error),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::Storage(msg) => write!(f, "Storage error: {}", msg),
            DatabaseError::Document(msg) => write!(f, "Document error: {}", msg),
            DatabaseError::Query(msg) => write!(f, "Query error: {}", msg),
            DatabaseError::Index(msg) => write!(f, "Index error: {}", msg),
            DatabaseError::Network(msg) => write!(f, "Network error: {}", msg),
            DatabaseError::Validation(msg) => write!(f, "Validation error: {}", msg),
            DatabaseError::Io(err) => write!(f, "IO error: {}", err),
            DatabaseError::Json(err) => write!(f, "JSON error: {}", err),
        }
    }
}

impl std::error::Error for DatabaseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DatabaseError::Io(err) => Some(err),
            DatabaseError::Json(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for DatabaseError {
    fn from(err: io::Error) -> DatabaseError {
        DatabaseError::Io(err)
    }
}

#[cfg(test)]
mod tests {
    
}
