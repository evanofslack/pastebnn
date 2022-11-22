use async_trait::async_trait;
extern crate redis;

use crate::db::Storer;
use crate::models;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde_json;

pub struct Redis {
    pub conn: ConnectionManager,
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
            hostname: "localhost".to_string(),
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

impl Redis {
    pub async fn new(conn_info: ConnInfo) -> Result<Self, &'static str> {
        // expected format -> redis://[<username>][:<password>@]<hostname>[:port][/<db>]
        let conn_url = format!(
            "redis://{}:{}@{}:{}",
            conn_info.username.unwrap_or_default(),
            conn_info.password.unwrap_or_default(),
            conn_info.hostname,
            conn_info.port
        );
        println!("{}", conn_url);
        match redis::Client::open(conn_url) {
            Ok(client) => match ConnectionManager::new(client.clone()).await {
                Ok(conn) => Ok(Redis { conn }),
                Err(err) => {
                    println!("{}", err);
                    Err("failed to connect to redis instance")
                }
            },
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
        let db = Redis::new(ConnInfo::default()).await.unwrap();
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
        let db = Redis::new(ConnInfo::default()).await.unwrap();
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
        let db = Redis::new(ConnInfo::default()).await.unwrap();
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
