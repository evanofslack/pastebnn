use async_trait::async_trait;
// use std::time::{SystemTime, UNIX_EPOCH};
extern crate redis;

use crate::db::Storer;
use crate::models;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde_json;

pub struct Redis {
    pub conn: ConnectionManager,
}

impl Redis {
    pub async fn new() -> Result<Self, &'static str> {
        match redis::Client::open("redis://127.0.0.1") {
            Ok(client) => {
                let conn = ConnectionManager::new(client.clone()).await.unwrap();
                return Ok(Redis { conn });
            }
            Err(_) => Err("failed to parse redis connection string"),
        }
    }
}

#[async_trait]
impl Storer for Redis {
    async fn get(&self, key: String) -> Result<models::Paste, &'static str> {
        let found_paste: models::Paste;
        match self.conn.clone().get::<_, String>(key).await {
            Ok(paste_str) => match serde_json::from_str::<models::Paste>(&paste_str) {
                Ok(paste) => {
                    found_paste = paste;
                }
                Err(err) => {
                    println!("{}", err);
                    return Err("Parse to JSON Failed");
                }
            },
            Err(err) => {
                println!("{}", err);
                return Err("paste not found");
            }
        }
        if found_paste.burn_on_read {
            self.delete(&found_paste.key).await?
        }
        Ok(found_paste)
    }

    async fn create(&self, paste: models::Paste) -> Result<(), &'static str> {
        let paste_str = serde_json::to_string(&paste).unwrap();
        let mut conn = self.conn.clone();

        // if paste has expiration, use redis command SETEX to set key with expiration
        // if paste does not have expiration, use redis command SET to set key normally
        match paste.expires {
            Some(time) => {
                match conn
                    .set_ex::<String, String, String>(
                        paste.key,
                        paste_str,
                        time.try_into().unwrap(),
                    )
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(err) => {
                        println!("{}", err);
                        Err("error creating paste in redis store")
                    }
                }
            }
            None => {
                match conn
                    .set::<String, String, String>(paste.key.clone(), paste_str)
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(err) => {
                        println!("{}", err);
                        Err("error creating paste in redis store")
                    }
                }
            }
        }
    }

    async fn delete(&self, key: &String) -> Result<(), &'static str> {
        match self.conn.clone().del::<String, i32>(key.to_string()).await {
            Ok(_) => Ok(()),
            Err(err) => {
                println!("{}", err);
                Err("paste not found")
            }
        }
    }

    async fn get_expired(&self) -> Vec<models::Paste> {
        // redis key/values pairs expire natively if set with SETEX
        let empty: Vec<models::Paste> = vec![];
        return empty;
    }
    async fn delete_expired(&self) -> Result<(), &'static str> {
        // redis key/values pairs expire natively if set with SETEX
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_and_get() {
        let db = Redis::new().await.unwrap();
        let paste = models::Paste::new(String::from("key"), String::from("text"), None, true);
        db.create(paste.clone())
            .await
            .expect("should create new paste");
        let resp = db
            .get(paste.clone().key)
            .await
            .expect("should return paste");

        assert_eq!(paste, resp);
    }

    #[tokio::test]
    async fn delete() {
        let db = Redis::new().await.unwrap();
        let paste = models::Paste::new(String::from("key"), String::from("text"), None, false);
        db.create(paste.clone())
            .await
            .expect("should create new paste");

        db.delete(&paste.clone().key)
            .await
            .expect("should delete paste");

        let resp = db.get(paste.clone().key).await;
        assert!(resp.is_err());
        if let Err(msg) = resp {
            assert_eq!(msg, "paste not found")
        }
    }

    #[tokio::test]
    async fn burn_on_read() {
        let db = Redis::new().await.unwrap();
        let paste = models::Paste::new(String::from("key"), String::from("text"), None, true);
        db.create(paste.clone())
            .await
            .expect("should create new paste");
        let resp = db
            .get(paste.clone().key)
            .await
            .expect("should return paste");
        assert_eq!(paste, resp);

        let resp = db.get(paste.clone().key).await;
        assert!(resp.is_err());
        if let Err(msg) = resp {
            assert_eq!(msg, "paste not found")
        }
    }
}
