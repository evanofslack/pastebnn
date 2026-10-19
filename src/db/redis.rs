use async_trait::async_trait;
extern crate redis;

use crate::db::{StoreError, Storer};
use crate::models;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde_json;

pub struct Redis {
    pub conn: ConnectionManager,
    max_size: usize,
}

#[derive(Debug)]
pub struct ConnInfo {
    hostname: String,
    port: i32,
    username: Option<String>,
    password: Option<String>,
}

impl Default for ConnInfo {
    fn default() -> Self {
        Self {
            hostname: "127.0.0.1".to_string(),
            port: 6379,
            username: None,
            password: None,
        }
    }
}

impl ConnInfo {
    pub fn new(
        hostname: String,
        port: i32,
        username: Option<String>,
        password: Option<String>,
    ) -> Self {
        Self {
            hostname,
            port,
            username,
            password,
        }
    }
}

fn internal(err: impl std::fmt::Display) -> StoreError {
    tracing::warn!("redis error: {}", err);
    StoreError::Internal(err.to_string())
}

impl Redis {
    pub async fn new(conn_info: ConnInfo, max_size: usize) -> Result<Self, StoreError> {
        // expected format -> redis://[<username>][:<password>@]<hostname>[:port][/<db>]
        let conn_url = format!(
            "redis://{}:{}@{}:{}",
            conn_info.username.unwrap_or_default(),
            conn_info.password.unwrap_or_default(),
            conn_info.hostname,
            conn_info.port
        );
        let client = redis::Client::open(conn_url).map_err(internal)?;
        let conn = ConnectionManager::new(client).await.map_err(internal)?;
        Ok(Redis { conn, max_size })
    }

    async fn fetch(&self, key: &str) -> Result<models::Paste, StoreError> {
        let paste_str: Option<String> = self.conn.clone().get(key).await.map_err(internal)?;
        let paste_str = paste_str.ok_or(StoreError::NotFound)?;
        serde_json::from_str(&paste_str).map_err(internal)
    }
}

#[async_trait]
impl Storer for Redis {
    async fn get(&self, key: &str) -> Result<models::Paste, StoreError> {
        let paste = self.fetch(key).await?;
        if paste.burn_on_read {
            // only the reader whose DEL removed the key gets to see the paste
            let removed: i32 = self.conn.clone().del(key).await.map_err(internal)?;
            if removed == 0 {
                return Err(StoreError::NotFound);
            }
        }
        Ok(paste)
    }

    async fn peek(&self, key: &str) -> Result<models::Paste, StoreError> {
        self.fetch(key).await
    }

    async fn create(&self, paste: models::Paste) -> Result<(), StoreError> {
        if paste.text.len() > self.max_size {
            return Err(StoreError::TooLarge);
        }

        let paste_str = serde_json::to_string(&paste).map_err(internal)?;

        // NX refuses to overwrite an existing key, EX lets redis expire the paste natively
        let mut cmd = redis::cmd("SET");
        cmd.arg(&paste.key).arg(paste_str).arg("NX");
        if let Some(expires) = paste.expires {
            let ttl_millis = expires.saturating_sub(models::now_millis());
            let ttl_secs = u64::try_from(ttl_millis.div_ceil(1000))
                .unwrap_or(u64::MAX)
                .max(1);
            cmd.arg("EX").arg(ttl_secs);
        }
        let reply: Option<String> = cmd
            .query_async(&mut self.conn.clone())
            .await
            .map_err(internal)?;
        match reply {
            Some(_) => Ok(()),
            None => Err(StoreError::AlreadyExists),
        }
    }

    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        let removed: i32 = self.conn.clone().del(key).await.map_err(internal)?;
        if removed == 0 {
            Err(StoreError::NotFound)
        } else {
            Ok(())
        }
    }

    async fn delete_expired(&self) -> Result<(), StoreError> {
        // redis key/values pairs expire natively if set with EX
        Ok(())
    }
}

#[cfg(all(test, feature = "integration-tests"))]
mod tests {
    use super::*;

    async fn test_db(max_size: usize) -> Redis {
        Redis::new(ConnInfo::default(), max_size).await.unwrap()
    }

    fn unique_key() -> String {
        format!("test-{}", models::generate_key())
    }

    #[tokio::test]
    async fn paste_too_large() {
        // paste new text is 4 bytes while max_size is 3 bytes
        let db = test_db(3).await;
        let paste = models::Paste::new(unique_key(), String::from("text"), None, true);
        assert_eq!(db.create(paste).await, Err(StoreError::TooLarge))
    }

    #[tokio::test]
    async fn create_and_get() {
        let db = test_db(1024).await;
        let paste = models::Paste::new(unique_key(), String::from("text"), None, true);
        db.create(paste.clone())
            .await
            .expect("should create new paste");
        let resp = db.get(&paste.key).await.expect("should return paste");

        assert_eq!(paste, resp);
    }

    #[tokio::test]
    async fn duplicate_key() {
        let db = test_db(1024).await;
        let key = unique_key();
        let first = models::Paste::new(key.clone(), String::from("first"), None, false);
        let second = models::Paste::new(key.clone(), String::from("second"), None, false);
        db.create(first).await.expect("should create new paste");

        assert_eq!(db.create(second).await, Err(StoreError::AlreadyExists));
        assert_eq!(db.get(&key).await.unwrap().text, "first");
        db.delete(&key).await.unwrap();
    }

    #[tokio::test]
    async fn expiry_sets_ttl() {
        let db = test_db(1024).await;
        let paste = models::Paste::new(unique_key(), String::from("text"), Some(600), false);
        db.create(paste.clone())
            .await
            .expect("should create new paste");

        let ttl: i64 = db.conn.clone().ttl(&paste.key).await.unwrap();
        assert!((590..=600).contains(&ttl), "unexpected ttl {}", ttl);
        db.delete(&paste.key).await.unwrap();
    }

    #[tokio::test]
    async fn delete() {
        let db = test_db(1024).await;
        let paste = models::Paste::new(unique_key(), String::from("text"), None, false);
        db.create(paste.clone())
            .await
            .expect("should create new paste");

        db.delete(&paste.key).await.expect("should delete paste");

        assert_eq!(db.get(&paste.key).await, Err(StoreError::NotFound));
        assert_eq!(db.delete(&paste.key).await, Err(StoreError::NotFound));
    }

    #[tokio::test]
    async fn burn_on_read() {
        let db = test_db(1024).await;
        let paste = models::Paste::new(unique_key(), String::from("text"), None, true);
        db.create(paste.clone())
            .await
            .expect("should create new paste");
        assert_eq!(db.peek(&paste.key).await.unwrap(), paste);
        let resp = db.get(&paste.key).await.expect("should return paste");
        assert_eq!(paste, resp);

        assert_eq!(db.get(&paste.key).await, Err(StoreError::NotFound));
    }
}
