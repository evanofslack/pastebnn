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
            // Ok(client) => match client.get_async_connection().await {
            //     Ok(mut conn) => Ok(Redis { conn }),
            //     Err(_) => Err("failed to make connection with redis client"),
            // },
        }
    }
}

#[async_trait]
impl Storer for Redis {
    async fn get(&self, key: String) -> Result<models::Paste, &'static str> {
        // let found_paste: models::Paste;
        // if let Some(paste) = self.db.read().unwrap().get(&key) {
        //     found_paste = paste.clone();
        // } else {
        //     return Err("paste not found");
        // }
        // if found_paste.burn_on_read {
        //     self.delete(&found_paste.key).await?;
        // }
        // return Ok(found_paste);
        unimplemented!();
    }

    async fn create(&self, paste: models::Paste) -> Result<(), &'static str> {
        let str_paste = serde_json::to_string(&paste).unwrap();
        match self
            .conn
            .clone()
            .set::<_, _, String>(paste.key.clone(), str_paste)
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => {
                println!("{}", err);
                Err("error creating paste in redis store")
            }
        }
    }

    async fn delete(&self, key: &String) -> Result<models::Paste, &'static str> {
        // if let Some(paste) = self.db.write().unwrap().remove(key) {
        //     return Ok(paste);
        // } else {
        //     return Err("paste not found");
        // }
        unimplemented!();
    }
    async fn get_expired(&self) -> Vec<models::Paste> {
        // let now = SystemTime::now()
        //     .duration_since(UNIX_EPOCH)
        //     .expect("Time went backwards")
        //     .as_millis();

        // let mut expired: Vec<models::Paste> = Vec::new();

        // for (_, paste) in self.db.read().unwrap().iter() {
        //     if let Some(expires) = paste.expires {
        //         if expires <= now {
        //             expired.push(paste.to_owned())
        //         }
        //     }
        // }
        // return expired;
        unimplemented!();
    }
    async fn delete_expired(&self) -> Result<(), &'static str> {
        // let expired = self.get_expired().await;
        // for paste in expired.iter() {
        //     self.delete(&paste.key).await?;
        // }
        // return Ok(());
        unimplemented!();
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

    //     #[tokio::test]
    //     async fn delete() {
    //         let db = InMemory::default();
    //         let paste = models::Paste::new(String::from("key"), String::from("text"), None, false);
    //         db.create(paste.clone())
    //             .await
    //             .expect("should create new paste");

    //         let resp = db
    //             .delete(&paste.clone().key)
    //             .await
    //             .expect("should return paste");
    //         assert_eq!(paste, resp);

    //         let resp = db.get(paste.clone().key).await;
    //         assert!(resp.is_err());
    //         if let Err(msg) = resp {
    //             assert_eq!(msg, "paste not found")
    //         }
    //     }

    //     #[tokio::test]
    //     async fn burn_on_read() {
    //         let db = InMemory::default();
    //         let paste = models::Paste::new(String::from("key"), String::from("text"), None, true);
    //         db.create(paste.clone())
    //             .await
    //             .expect("should create new paste");
    //         let resp = db
    //             .get(paste.clone().key)
    //             .await
    //             .expect("should return paste");
    //         assert_eq!(paste, resp);

    //         let resp = db.get(paste.clone().key).await;
    //         assert!(resp.is_err());
    //         if let Err(msg) = resp {
    //             assert_eq!(msg, "paste not found")
    //         }
    //     }
}
