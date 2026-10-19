use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;

use crate::db::{StoreError, Storer};
use crate::models;

#[derive(Default)]
struct Inner {
    pastes: HashMap<String, models::Paste>,
    total_bytes: usize,
}

impl Inner {
    fn remove(&mut self, key: &str) -> Option<models::Paste> {
        let paste = self.pastes.remove(key)?;
        self.total_bytes -= paste.text.len();
        Some(paste)
    }
}

pub struct InMemory {
    inner: RwLock<Inner>,
    max_size: usize,
    max_total_bytes: usize,
}

impl InMemory {
    pub async fn new(max_size: usize, max_total_bytes: usize) -> Result<Self, StoreError> {
        Ok(InMemory {
            inner: RwLock::new(Inner::default()),
            max_size,
            max_total_bytes,
        })
    }

    #[cfg(test)]
    fn total_bytes(&self) -> usize {
        self.inner.read().unwrap().total_bytes
    }
}

#[async_trait]
impl Storer for InMemory {
    async fn get(&self, key: &str) -> Result<models::Paste, StoreError> {
        let now = models::now_millis();
        {
            let inner = self.inner.read().unwrap();
            match inner.pastes.get(key) {
                Some(paste) if paste.is_expired(now) => return Err(StoreError::NotFound),
                Some(paste) if !paste.burn_on_read => return Ok(paste.clone()),
                Some(_) => {}
                None => return Err(StoreError::NotFound),
            }
        }
        // burn on read: re-check under the write lock so only one reader gets the paste
        let mut inner = self.inner.write().unwrap();
        let (burn, expired) = match inner.pastes.get(key) {
            Some(paste) => (paste.burn_on_read, paste.is_expired(now)),
            None => return Err(StoreError::NotFound),
        };
        if expired {
            return Err(StoreError::NotFound);
        }
        if burn {
            inner.remove(key).ok_or(StoreError::NotFound)
        } else {
            Ok(inner.pastes[key].clone())
        }
    }

    async fn peek(&self, key: &str) -> Result<models::Paste, StoreError> {
        let now = models::now_millis();
        match self.inner.read().unwrap().pastes.get(key) {
            Some(paste) if !paste.is_expired(now) => Ok(paste.clone()),
            _ => Err(StoreError::NotFound),
        }
    }

    async fn create(&self, paste: models::Paste) -> Result<(), StoreError> {
        let size = paste.text.len();
        if size > self.max_size {
            return Err(StoreError::TooLarge);
        }
        let now = models::now_millis();
        let mut inner = self.inner.write().unwrap();
        // an expired paste awaiting purge should not block reuse of its key
        if matches!(inner.pastes.get(&paste.key), Some(existing) if existing.is_expired(now)) {
            inner.remove(&paste.key);
        }
        if inner.pastes.contains_key(&paste.key) {
            return Err(StoreError::AlreadyExists);
        }
        if inner.total_bytes + size > self.max_total_bytes {
            return Err(StoreError::StorageFull);
        }
        inner.total_bytes += size;
        inner.pastes.insert(paste.key.clone(), paste);
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        match self.inner.write().unwrap().remove(key) {
            Some(_) => Ok(()),
            None => Err(StoreError::NotFound),
        }
    }

    async fn delete_expired(&self) -> Result<(), StoreError> {
        let now = models::now_millis();
        let mut inner = self.inner.write().unwrap();
        let expired: Vec<String> = inner
            .pastes
            .values()
            .filter(|paste| paste.is_expired(now))
            .map(|paste| paste.key.clone())
            .collect();
        for key in expired {
            inner.remove(&key);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn paste_too_large() {
        // paste new text is 4 bytes while max_size is 3 bytes
        let db = InMemory::new(3, 1024).await.unwrap();
        let paste = models::Paste::new(String::from("key"), String::from("text"), None, true);
        assert_eq!(db.create(paste).await, Err(StoreError::TooLarge))
    }

    #[tokio::test]
    async fn create_and_get() {
        let db = InMemory::new(1024, 1024).await.unwrap();
        let paste = models::Paste::new(String::from("key"), String::from("text"), None, true);
        db.create(paste.clone())
            .await
            .expect("should create new paste");
        let resp = db.get(&paste.key).await.expect("should return paste");

        assert_eq!(paste, resp);
    }

    #[tokio::test]
    async fn duplicate_key() {
        let db = InMemory::new(1024, 1024).await.unwrap();
        let first = models::Paste::new(String::from("key"), String::from("first"), None, false);
        let second = models::Paste::new(String::from("key"), String::from("second"), None, false);
        db.create(first).await.expect("should create new paste");

        assert_eq!(db.create(second).await, Err(StoreError::AlreadyExists));
        assert_eq!(db.get("key").await.unwrap().text, "first");
    }

    #[tokio::test]
    async fn delete() {
        let db = InMemory::new(1024, 1024).await.unwrap();
        let paste = models::Paste::new(String::from("key"), String::from("text"), None, false);
        db.create(paste.clone())
            .await
            .expect("should create new paste");

        db.delete(&paste.key).await.expect("should delete paste");

        assert_eq!(db.get(&paste.key).await, Err(StoreError::NotFound));
        assert_eq!(db.total_bytes(), 0);
    }

    #[tokio::test]
    async fn storage_full() {
        let db = InMemory::new(1024, 8).await.unwrap();
        let first = models::Paste::new(String::from("a"), String::from("12345"), None, false);
        let second = models::Paste::new(String::from("b"), String::from("12345"), None, false);
        db.create(first).await.expect("should create new paste");

        assert_eq!(
            db.create(second.clone()).await,
            Err(StoreError::StorageFull)
        );

        // freeing space allows new pastes again
        db.delete("a").await.expect("should delete paste");
        db.create(second).await.expect("should create new paste");
        assert_eq!(db.total_bytes(), 5);
    }

    #[tokio::test]
    async fn expired_not_returned() {
        let db = InMemory::new(1024, 1024).await.unwrap();
        let mut paste = models::Paste::new(String::from("key"), String::from("text"), None, false);
        paste.expires = Some(paste.created - 1);
        db.create(paste.clone())
            .await
            .expect("should create new paste");

        assert_eq!(db.get("key").await, Err(StoreError::NotFound));
        assert_eq!(db.peek("key").await, Err(StoreError::NotFound));

        // key of expired paste can be reused before purge
        let fresh = models::Paste::new(String::from("key"), String::from("new"), None, false);
        db.create(fresh)
            .await
            .expect("should replace expired paste");
        assert_eq!(db.total_bytes(), 3);
    }

    #[tokio::test]
    async fn delete_expired() {
        let db = InMemory::new(1024, 1024).await.unwrap();
        let mut expired =
            models::Paste::new(String::from("old"), String::from("text"), None, false);
        expired.expires = Some(expired.created - 1);
        let live = models::Paste::new(String::from("new"), String::from("text"), Some(60), false);
        db.create(expired).await.unwrap();
        db.create(live).await.unwrap();

        db.delete_expired().await.unwrap();

        assert_eq!(db.total_bytes(), 4);
        assert!(db.get("new").await.is_ok());
    }

    #[tokio::test]
    async fn burn_on_read() {
        let db = InMemory::new(1024, 1024).await.unwrap();
        let paste = models::Paste::new(String::from("key"), String::from("text"), None, true);
        db.create(paste.clone())
            .await
            .expect("should create new paste");

        // peek does not burn
        assert_eq!(db.peek(&paste.key).await.unwrap(), paste);

        let resp = db.get(&paste.key).await.expect("should return paste");
        assert_eq!(paste, resp);

        assert_eq!(db.get(&paste.key).await, Err(StoreError::NotFound));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn burn_on_read_concurrent() {
        let db = Arc::new(InMemory::new(1024, 1024).await.unwrap());
        let paste = models::Paste::new(String::from("key"), String::from("text"), None, true);
        db.create(paste).await.expect("should create new paste");

        let readers: Vec<_> = (0..16)
            .map(|_| {
                let db = db.clone();
                tokio::spawn(async move { db.get("key").await.is_ok() })
            })
            .collect();
        let mut successes = 0;
        for reader in readers {
            if reader.await.unwrap() {
                successes += 1;
            }
        }
        assert_eq!(successes, 1);
    }
}
