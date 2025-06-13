use crate::error::DatabaseError;

#[allow(dead_code)]
type DbResult<T> = Result<T, DatabaseError>;
