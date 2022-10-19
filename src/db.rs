use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use async_trait::async_trait;

use super::models;


#[async_trait]
pub trait Storer {
    async fn get(&self, key: String) -> Result<models::Paste, &'static str>;
    async fn create(&self, paste: models::Paste) -> Result<(), &'static str>;
    async fn delete(&self, key: &String) -> Result<models::Paste, &'static str>;
    async fn get_expired(&self) -> Vec<models::Paste>;
    async fn delete_expired(&self) -> Result<(), &'static str>;
}

#[derive(Default)]
pub struct InMemory {
    pub db: RwLock<HashMap<String, models::Paste>>
}


#[async_trait]
impl Storer for InMemory {
    async fn get(&self, key: String) -> Result<models::Paste, &'static str> {

        let found_paste: models::Paste;
        if let Some(paste) = self.db.read().unwrap().get(&key) {
            found_paste = paste.clone();
        } else {
            return Err("no paste found")
        }
        if found_paste.burn_on_read {
            self.delete(&found_paste.key).await?;
        }
        return Ok(found_paste)
    }

    async fn create(&self, paste: models::Paste) -> Result<(), &'static str>{
        self.db.write().unwrap().insert(paste.key.clone(), paste.clone());
        return Ok(())

    }
    async fn delete(&self, key: &String) -> Result<models::Paste, &'static str>{
        if let Some(paste) = self.db.write().unwrap().remove(key) {
            return Ok(paste)
        } else {
            return Err("Paste not found")
        }
    }
    async fn get_expired(&self) -> Vec<models::Paste>{
        let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();

        let mut expired: Vec<models::Paste> = Vec::new();

        for (_, paste) in self.db.read().unwrap().iter() {
            if let Some(expires) = paste.expires {
                if expires <= now {
                    expired.push(paste.to_owned())
                }
            }
        }
        return expired
    }
    async fn delete_expired(&self) -> Result<(), &'static str>{
        let expired = self.get_expired().await;
        for paste in expired.iter() {
            self.delete(&paste.key).await?;
        }
        return Ok(())
    }
}