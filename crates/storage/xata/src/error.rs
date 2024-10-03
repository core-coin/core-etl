use std::{error::Error, pin::Pin};

use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum XataStorageError {
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),
}

impl From<XataStorageError> for Pin<Box<dyn Error + Send + Sync>> {
    fn from(err: XataStorageError) -> Self {
        Pin::from(Box::new(err))
    }
}