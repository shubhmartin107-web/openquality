use rand::Rng;
use sha2::{Digest, Sha256};

const KEY_PREFIX: &str = "oq_";

pub struct ApiKeyPair {
    pub raw_key: String,
    pub key_hash: String,
}

pub fn generate_api_key() -> ApiKeyPair {
    let random_bytes: [u8; 32] = rand::thread_rng().r#gen();
    let raw_key = format!("{}{}", KEY_PREFIX, hex::encode(random_bytes));
    let key_hash = hash_api_key(&raw_key);
    ApiKeyPair { raw_key, key_hash }
}

pub fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn validate_api_key_format(key: &str) -> bool {
    key.starts_with(KEY_PREFIX) && key.len() == 67
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_generation() {
        let pair = generate_api_key();
        assert!(pair.raw_key.starts_with("oq_"));
        assert_eq!(pair.raw_key.len(), 67);
        assert_eq!(pair.key_hash, hash_api_key(&pair.raw_key));
    }

    #[test]
    fn test_api_key_validation() {
        let pair = generate_api_key();
        assert!(validate_api_key_format(&pair.raw_key));
        assert!(!validate_api_key_format("invalid-key"));
    }
}
