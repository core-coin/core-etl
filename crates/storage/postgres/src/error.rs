use std::{error::Error, pin::Pin};

use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum PostgresStorageError {
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),
}

impl From<PostgresStorageError> for Pin<Box<dyn Error + Send + Sync>> {
    fn from(err: PostgresStorageError) -> Self {
        Pin::from(Box::new(err))
    }
}
