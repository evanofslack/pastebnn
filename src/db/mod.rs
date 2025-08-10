use async_trait::async_trait;
use std::time::Duration;
use tokio::time;

use crate::models;
pub mod inmemory;
pub mod redis;

#[async_trait]
pub trait Storer: Sync {
    // async fn new(&self) -> Self;
    async fn get(&self, key: String) -> Result<models::Paste, &'static str>;
    async fn create(&self, paste: models::Paste) -> Result<(), &'static str>;
    async fn delete(&self, key: &str) -> Result<(), &'static str>;
    async fn get_expired(&self) -> Vec<models::Paste>;

    async fn delete_expired(&self) -> Result<(), &'static str> {
        for paste in self.get_expired().await.iter() {
            self.delete(&paste.key).await?;
        }
        return Ok(());
    }
    async fn delete_periodically(&self, period_seconds: u64) -> Result<(), &'static str> {
        let mut interval = time::interval(Duration::from_secs(period_seconds));
        tracing::debug!("Deleting expired pastes every {} seconds", period_seconds);
        loop {
            interval.tick().await;
            self.delete_expired().await?
        }
    }
}
