
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::time::{Duration, SystemTime};

#[derive(Deserialize)]
pub struct CreatePaste {
    pub key: String,
    pub text: String,
    pub expires: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Paste {
    pub id: Uuid,
    pub time_created: SystemTime,
    pub expires: Option<Duration>,
    pub key: String,
    pub text: String,
}

impl Paste {
    pub fn new(key: String, text: String, expires_seconds: Option<u64>) -> Self {
        let expires: Option<Duration>= match expires_seconds {
            Some(time) =>{Some(Duration::from_secs(time))},
            None => {None},
        };
        Self { id: Uuid::new_v4(), time_created: SystemTime::now(), expires: expires, key: key, text:text }
    }
}