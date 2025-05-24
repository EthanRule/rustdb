use crate::error::DatabaseError;

type DbResult<T> = Result<T, DatabaseError>;
