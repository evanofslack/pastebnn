use redis::{
    from_redis_value, ErrorKind, FromRedisValue, RedisResult, RedisWrite, ToRedisArgs, Value,
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

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
    pub fn new(
        key: String,
        text: String,
        seconds_until_expire: Option<u128>,
        burn_on_read: bool,
    ) -> Self {
        let created = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();

        let expires: Option<u128> = match seconds_until_expire {
            Some(time) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis();

                let additional = time * 1000;
                Some(now + additional)
            }
            None => None,
        };
        Self {
            id: Uuid::new_v4(),
            created: created,
            expires: expires,
            burn_on_read: burn_on_read,
            key: key,
            text: text,
        }
    }
}

// https://users.rust-lang.org/t/how-impl-a-trait-of-fromredisvalue-for-more-structs/67532/3
impl FromRedisValue for Paste {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        let json_str: String = from_redis_value(v)?;
        println!("{}", json_str);
        let result: Self = match serde_json::from_str(&json_str) {
            Ok(v) => v,
            Err(err) => {
                println!("{}", err);
                return Err((ErrorKind::TypeError, "Parse to JSON Failed").into());
            }
        };
        Ok(result)
    }
}

impl ToRedisArgs for Paste {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        out.write_arg(&serde_json::to_vec(&self).unwrap()[..])
    }
}
