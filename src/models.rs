use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Length of server generated paste keys (~59 bits of entropy)
pub const GENERATED_KEY_LEN: usize = 10;
/// Max length of user chosen paste keys
pub const MAX_KEY_LEN: usize = 64;
const DELETE_TOKEN_LEN: usize = 32;

#[derive(Deserialize)]
pub struct CreatePaste {
    pub key: Option<String>,
    pub text: String,
    pub seconds_until_expire: Option<u64>,
    #[serde(default)]
    pub burn_on_read: bool,
}

/// Paste as persisted in the store, including its secret delete token
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Paste {
    pub id: Uuid,
    pub created: u128,         // milliseconds since epoch
    pub expires: Option<u128>, // milliseconds since epoch
    pub burn_on_read: bool,
    pub key: String,
    pub text: String,
    // pastes stored before delete tokens existed have none and cannot be deleted
    #[serde(default)]
    pub delete_token: String,
}

impl Paste {
    pub fn new(
        key: String,
        text: String,
        seconds_until_expire: Option<u64>,
        burn_on_read: bool,
    ) -> Self {
        let created = now_millis();
        let expires = seconds_until_expire.map(|secs| created + u128::from(secs) * 1000);
        Self {
            id: Uuid::new_v4(),
            created,
            expires,
            burn_on_read,
            key,
            text,
            delete_token: random_string(DELETE_TOKEN_LEN),
        }
    }

    pub fn is_expired(&self, now: u128) -> bool {
        matches!(self.expires, Some(expires) if expires <= now)
    }
}

/// Public view of a paste, without the delete token
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PasteView {
    pub id: Uuid,
    pub created: u128,
    pub expires: Option<u128>,
    pub burn_on_read: bool,
    pub key: String,
    pub text: String,
}

impl From<&Paste> for PasteView {
    fn from(paste: &Paste) -> Self {
        Self {
            id: paste.id,
            created: paste.created,
            expires: paste.expires,
            burn_on_read: paste.burn_on_read,
            key: paste.key.clone(),
            text: paste.text.clone(),
        }
    }
}

/// Response to paste creation, the only time the delete token is revealed
#[derive(Debug, Serialize, Deserialize)]
pub struct CreatedPaste {
    #[serde(flatten)]
    pub paste: PasteView,
    pub delete_token: String,
}

impl From<&Paste> for CreatedPaste {
    fn from(paste: &Paste) -> Self {
        Self {
            paste: PasteView::from(paste),
            delete_token: paste.delete_token.clone(),
        }
    }
}

pub fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
}

pub fn generate_key() -> String {
    random_string(GENERATED_KEY_LEN)
}

fn random_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

/// User chosen keys must be 1-64 chars of [A-Za-z0-9_-]
pub fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= MAX_KEY_LEN
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

/// Constant time comparison of a provided delete token against the stored one
pub fn token_matches(stored: &str, provided: &str) -> bool {
    if stored.is_empty() || stored.len() != provided.len() {
        return false;
    }
    stored
        .bytes()
        .zip(provided.bytes())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_validation() {
        assert!(valid_key("my-paste_1"));
        assert!(valid_key(&"a".repeat(MAX_KEY_LEN)));
        assert!(!valid_key(""));
        assert!(!valid_key(&"a".repeat(MAX_KEY_LEN + 1)));
        assert!(!valid_key("../etc"));
        assert!(!valid_key("has space"));
    }

    #[test]
    fn generated_keys_are_valid() {
        let key = generate_key();
        assert_eq!(key.len(), GENERATED_KEY_LEN);
        assert!(valid_key(&key));
    }

    #[test]
    fn token_matching() {
        assert!(token_matches("abc", "abc"));
        assert!(!token_matches("abc", "abd"));
        assert!(!token_matches("abc", "ab"));
        assert!(!token_matches("", ""));
    }

    #[test]
    fn view_omits_token() {
        let paste = Paste::new("key".into(), "text".into(), None, false);
        let json = serde_json::to_value(PasteView::from(&paste)).unwrap();
        assert!(json.get("delete_token").is_none());
    }
}
