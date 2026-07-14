use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::rbac::Role;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub workspace_id: Uuid,
    pub email: String,
    pub role: Role,
    pub exp: usize,
    pub iat: usize,
}

pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: String,
    expiration_hours: i64,
}

impl JwtManager {
    pub fn new(secret: &[u8], expiration_hours: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            issuer: "openquality".to_string(),
            expiration_hours,
        }
    }

    pub fn create_token(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
        email: &str,
        role: &Role,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id,
            workspace_id,
            email: email.to_string(),
            role: role.clone(),
            iat: now.timestamp() as usize,
            exp: (now + chrono::Duration::hours(self.expiration_hours)).timestamp() as usize,
        };
        encode(&Header::default(), &claims, &self.encoding_key)
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let mut validation = Validation::default();
        validation.set_issuer(&[&self.issuer]);
        validation.validate_exp = true;
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;
        Ok(token_data.claims)
    }

    pub fn refresh_token(&self, claims: &Claims) -> Result<String, jsonwebtoken::errors::Error> {
        self.create_token(claims.sub, claims.workspace_id, &claims.email, &claims.role)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_roundtrip() {
        let manager = JwtManager::new(b"test-secret-12345", 24);
        let user_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();
        let token = manager
            .create_token(user_id, ws_id, "test@example.com", &Role::Admin)
            .unwrap();
        let claims = manager.validate_token(&token).unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.role, Role::Admin);
    }

    #[test]
    fn test_jwt_invalid_token() {
        let manager = JwtManager::new(b"test-secret-12345", 24);
        let result = manager.validate_token("invalid-token");
        assert!(result.is_err());
    }
}
