use std::collections::HashMap;
use std::sync::RwLock;
use async_trait::async_trait;

use super::models;


#[async_trait]
pub trait Storer {
    async fn get(&self, key: String) -> Result<&models::Paste, &'static str>;
    async fn create(&self, paste: models::Paste) -> Result<(), &'static str>;
    async fn delete(&self, key: String) -> Result<models::Paste, &'static str>;
}

#[derive(Default)]
pub struct InMemory {
    pub db: RwLock<HashMap<String, models::Paste>>
}


#[async_trait]
impl Storer for InMemory {
    async fn get(&self, key: String) -> Result<&models::Paste, &'static str> {
        if let Some(paste) = self.db.write().unwrap().get(&key) {
            return Ok(paste)
        } else {
            return Err("no paste found")
        }
    }

    async fn create(&self, paste: models::Paste) -> Result<(), &'static str>{
        self.db.write().unwrap().insert(paste.key.clone(), paste.clone());
        return Ok(())

    }
    async fn delete(&self, key: String) -> Result<models::Paste, &'static str>{
        if let Some(paste) = self.db.write().unwrap().remove(&key) {
            return Ok(paste)
        } else {
            return Err("no paste found")
        }
    }
}