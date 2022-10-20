
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
pub struct CreatePaste {
    pub key: String,
    pub text: String,
    pub seconds_until_expire: Option<u128>,
    pub burn_on_read: bool,
}


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Paste {
    pub id: Uuid,
    pub created: u128,         // milliseconds since epoch
    pub expires: Option<u128>, // milliseconds since epoch
    pub burn_on_read: bool,
    pub key: String,
    pub text: String,
}

impl Paste {
    pub fn new(key: String, text: String, seconds_until_expire: Option<u128>, burn_on_read: bool) -> Self {
        let created = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();

        let expires: Option<u128>= match seconds_until_expire {
            Some(time) =>{
                let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis();

                let additional = time * 1000;
                Some(now + additional)
            },
            None => {None},
        };
        Self { id: Uuid::new_v4(), created: created, expires: expires, burn_on_read: burn_on_read, key: key, text:text }
    }
}