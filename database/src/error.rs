use bincode;
use serde_json;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum DatabaseError {
    Storage(String),
    Document(String),
    Query(String),
    Index(String),
    Network(String),
    Validation(String),
    InvalidChecksum,
    Io(io::Error),
    Json(serde_json::Error),
    Bincode(bincode::Error),
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
            DatabaseError::InvalidChecksum => write!(f, "Invalid page checksum"),
            DatabaseError::Io(err) => write!(f, "IO error: {}", err),
            DatabaseError::Json(err) => write!(f, "JSON error: {}", err),
            DatabaseError::Bincode(err) => write!(f, "Bincode error: {}", err),
        }
    }
}

impl std::error::Error for DatabaseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DatabaseError::Io(err) => Some(err),
            DatabaseError::Json(err) => Some(err),
            DatabaseError::Bincode(err) => Some(err),
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
    use super::*;

    #[test]
    fn test_error_conversion_and_display() {
        let io_error = io::Error::new(io::ErrorKind::Other, "disk full");
        let db_error: DatabaseError = io_error.into();

        match db_error {
            DatabaseError::Io(ref err) => assert_eq!(err.to_string(), "disk full"),
            _ => panic!("Expected DatabaseError::Io variant"),
        }

        assert_eq!(format!("{}", db_error), "IO error: disk full");
    }

    #[test]
    fn test_json_error_conversion_and_display() {
        let json_error = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let db_error = DatabaseError::Json(json_error);

        assert!(format!("{}", db_error).starts_with("JSON error:"));
    }

    #[test]
    fn test_storage_error_display() {
        let storage_error = DatabaseError::Storage("Failed to connect to storage".to_string());
        assert_eq!(
            format!("{}", storage_error),
            "Storage error: Failed to connect to storage"
        );
    }

    #[test]
    fn test_document_error_display() {
        let document_error = DatabaseError::Document("Document not found".to_string());
        assert_eq!(
            format!("{}", document_error),
            "Document error: Document not found"
        );
    }

    #[test]
    fn test_query_error_display() {
        let query_error = DatabaseError::Query("Invalid query syntax".to_string());
        assert_eq!(
            format!("{}", query_error),
            "Query error: Invalid query syntax"
        );
    }

    #[test]
    fn test_index_error_display() {
        let index_error = DatabaseError::Index("Index creation failed".to_string());
        assert_eq!(
            format!("{}", index_error),
            "Index error: Index creation failed"
        );
    }

    #[test]
    fn test_network_error_display() {
        let network_error = DatabaseError::Network("Network timeout".to_string());
        assert_eq!(
            format!("{}", network_error),
            "Network error: Network timeout"
        );
    }

    #[test]
    fn test_validation_error_display() {
        let validation_error = DatabaseError::Validation("Invalid data format".to_string());
        assert_eq!(
            format!("{}", validation_error),
            "Validation error: Invalid data format"
        );
    }
}
