use std::collections::HashMap;

use super::models;

pub trait PasteService {
    fn get(&self, key: String) -> Result<models::Paste, Err>;
    fn create(&self, paste: models::Paste) -> Result<(), Err>;
    fn delete(&self, key: String) -> Result<models::Paste, Err>;
}

pub struct InMemory {
    db: HashMap<String, models::Paste>
}

impl PasteService for InMemory {
    fn get(&self, key: String) -> Result<models::Paste, Err> {
        if let Some(paste) = self.db.get(&key) {
            return Ok(paste)
        } else {
            return Err("no paste found")
        }
        // let paste = self.db.get(&key);
        // match paste {
        //     Some(paste) => {Ok(paste)}
        //     None() => {Err("no paste found for key")}
        // }
    }

    fn create(&self, paste: models::Paste) -> Result<(), Err>{
        let paste = self.db.insert(paste.key.clone(), paste.clone());
        return Ok(())

    }
    fn delete(&self, key: String) -> Result<models::Paste, Err>{
        if let Some(paste) = self.db.remove(&key) {
            return Ok(paste)
        } else {
            return Err("no paste found")
        }
    }
}