use crate::config::Config;
use crate::models::{Claims, User};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

pub fn generate_token(user_id: Uuid, config: &Config) -> anyhow::Result<String> {
    let now = Utc::now();
    let expires_at = now + Duration::hours(config.jwt_expiry_hours);
    
    let claims = Claims {
        sub: user_id,
        exp: expires_at.timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )?;
    
    Ok(token)
}

pub fn verify_token(token: &str, config: &Config) -> anyhow::Result<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )?;
    
    Ok(token_data.claims)
}

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let hashed = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;
    Ok(hashed)
}

pub fn verify_password(password: &str, hash: &str) -> anyhow::Result<bool> {
    let valid = bcrypt::verify(password, hash)?;
    Ok(valid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let password = "test_password123";
        let hash = hash_password(password).unwrap();
        
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }
}
