use async_trait::async_trait;
use std::fmt;
use std::time::Duration;
use tokio::time;

use crate::models;
pub mod inmemory;
pub mod redis;

#[derive(Debug, PartialEq, Eq)]
pub enum StoreError {
    NotFound,
    AlreadyExists,
    TooLarge,
    StorageFull,
    Internal(String),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::NotFound => write!(f, "paste not found"),
            StoreError::AlreadyExists => write!(f, "paste key already exists"),
            StoreError::TooLarge => write!(f, "paste is larger than maximum allowed size"),
            StoreError::StorageFull => write!(f, "storage is full"),
            StoreError::Internal(msg) => write!(f, "internal store error: {}", msg),
        }
    }
}

#[async_trait]
pub trait Storer: Sync {
    /// Get a paste, deleting it if it is burn on read
    async fn get(&self, key: &str) -> Result<models::Paste, StoreError>;
    /// Get a paste without triggering burn on read
    async fn peek(&self, key: &str) -> Result<models::Paste, StoreError>;
    /// Create a paste, failing if the key is already taken
    async fn create(&self, paste: models::Paste) -> Result<(), StoreError>;
    async fn delete(&self, key: &str) -> Result<(), StoreError>;
    async fn delete_expired(&self) -> Result<(), StoreError>;

    async fn delete_periodically(&self, period_seconds: u64) {
        let mut interval = time::interval(Duration::from_secs(period_seconds));
        tracing::debug!("Deleting expired pastes every {} seconds", period_seconds);
        loop {
            interval.tick().await;
            if let Err(error) = self.delete_expired().await {
                tracing::error!("failed to delete expired pastes: {}", error)
            }
        }
    }
}
