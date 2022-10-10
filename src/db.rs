use std::collections::HashMap;

use super::models;

pub trait PasteService {
    fn get(&mut self, key: String) -> Result<&models::Paste, &'static str>;
    fn create(&mut self, paste: models::Paste) -> Result<(), &'static str>;
    fn delete(&mut self, key: String) -> Result<models::Paste, &'static str>;
}

#[derive(Default)]
pub struct InMemory {
    pub db: HashMap<String, models::Paste>
}

impl PasteService for InMemory {
    fn get(&mut self, key: String) -> Result<&models::Paste, &'static str> {
        if let Some(paste) = self.db.get(&key) {
            return Ok(paste)
        } else {
            return Err("no paste found")
        }
    }

    fn create(&mut self, paste: models::Paste) -> Result<(), &'static str>{
        self.db.insert(paste.key.clone(), paste.clone());
        return Ok(())

    }
    fn delete(&mut self, key: String) -> Result<models::Paste, &'static str>{
        if let Some(paste) = self.db.remove(&key) {
            return Ok(paste)
        } else {
            return Err("no paste found")
        }
    }
}