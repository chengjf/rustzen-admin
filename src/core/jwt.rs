use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing;

use crate::core::config::CONFIG;

/// JWT Configuration, loaded from environment variables.
///
/// This struct holds the essential settings for JWT generation and validation,
/// including the secret key and token expiration duration.
#[derive(Debug)]
pub struct JwtConfig {
    /// The secret key used for signing and verifying tokens.
    pub secret: String,
    /// The duration in seconds for which a token is valid.
    pub expiration: i64,
}

/// The global JWT configuration instance, initialized lazily.
///
/// Reads `JWT_SECRET` and `JWT_EXPIRATION` from environment variables.
/// Provides default values and logs warnings if they are not set, which is
/// crucial for security and debugging during development.
pub static JWT_CONFIG: Lazy<JwtConfig> = Lazy::new(|| {
    let secret = CONFIG.jwt_secret.clone();
    let expiration = CONFIG.jwt_expiration;

    tracing::info!("JWT initialized: expiration={}s, secret_len={}", expiration, secret.len());
    JwtConfig { secret, expiration }
});

/// Represents the claims in the JWT payload.
///
/// These claims contain the token's subject information and metadata.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// The subject of the token, typically the user ID.
    pub user_id: i64,
    /// The username associated with the token.
    pub username: String,
    /// Expiration time (as a Unix timestamp).
    pub exp: usize,
    /// Issued at time (as a Unix timestamp).
    pub iat: usize,
}

/// Generates a new JWT for a given user.
///
/// # Arguments
///
/// * `user_id` - The ID of the user for whom the token is generated.
/// * `username` - The username of the user.
///
/// # Errors
///
/// Returns a `jsonwebtoken::errors::Error` if token generation fails.
pub fn generate_token(user_id: i64, username: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = (now + Duration::seconds(JWT_CONFIG.expiration)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims { user_id, username: username.to_string(), exp, iat };

    tracing::debug!("Generating token for user '{}' (ID: {})", username, user_id);

    encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_CONFIG.secret.as_bytes()))
}

/// Verifies a JWT and returns the claims if valid.
///
/// # Arguments
///
/// * `token` - The JWT string to verify.
///
/// # Errors
///
/// Returns a `jsonwebtoken::errors::Error` if the token is invalid, expired,
/// or if verification otherwise fails.
pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_CONFIG.secret.as_bytes()),
        &validation,
    )?;

    tracing::trace!("Successfully verified token for user '{}'", token_data.claims.username);
    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{EncodingKey, Header, encode};

    /// Round-trip: generate_token → verify_token should return the original claims.
    #[test]
    fn generate_then_verify_round_trip() {
        let token = generate_token(42, "alice").expect("token generation must succeed");
        assert!(!token.is_empty());

        let claims = verify_token(&token).expect("verify must succeed on a freshly generated token");
        assert_eq!(claims.user_id, 42);
        assert_eq!(claims.username, "alice");
        assert!(claims.exp > claims.iat, "expiry must be after issue time");
    }

    /// Claims fields must exactly match the values passed to generate_token.
    #[test]
    fn claims_fields_match_input() {
        let token = generate_token(99, "bob").unwrap();
        let claims = verify_token(&token).unwrap();
        assert_eq!(claims.user_id, 99);
        assert_eq!(claims.username, "bob");
    }

    /// A random garbage string is not a valid JWT.
    #[test]
    fn verify_rejects_garbage_string() {
        let result = verify_token("this.is.not.a.jwt");
        assert!(result.is_err(), "garbage token must be rejected");
    }

    /// An empty string is not a valid JWT.
    #[test]
    fn verify_rejects_empty_string() {
        let result = verify_token("");
        assert!(result.is_err());
    }

    /// A token signed with a different secret must be rejected.
    #[test]
    fn verify_rejects_token_with_wrong_secret() {
        let wrong_key = EncodingKey::from_secret(b"completely-different-secret-xyz");
        let now = Utc::now();
        let claims = Claims {
            user_id: 1,
            username: "attacker".to_string(),
            exp: (now + Duration::hours(1)).timestamp() as usize,
            iat: now.timestamp() as usize,
        };
        let forged = encode(&Header::default(), &claims, &wrong_key)
            .expect("encoding with wrong key should succeed");

        let result = verify_token(&forged);
        assert!(result.is_err(), "token signed with wrong secret must be rejected");
    }

    /// A token whose `exp` is in the past must be rejected.
    #[test]
    fn verify_rejects_expired_token() {
        let now = Utc::now();
        let claims = Claims {
            user_id: 7,
            username: "expired_user".to_string(),
            exp: (now - Duration::seconds(1)).timestamp() as usize,
            iat: (now - Duration::hours(1)).timestamp() as usize,
        };
        let expired = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(JWT_CONFIG.secret.as_bytes()),
        )
        .expect("encoding expired token must succeed");

        let result = verify_token(&expired);
        assert!(result.is_err(), "expired token must be rejected");
    }

    /// A structurally valid JWT with a tampered payload must be rejected.
    #[test]
    fn verify_rejects_tampered_payload() {
        let token = generate_token(1, "original").unwrap();
        // Replace the payload segment with a base64-encoded different payload
        let parts: Vec<&str> = token.splitn(3, '.').collect();
        assert_eq!(parts.len(), 3, "JWT must have 3 parts");
        let tampered = format!("{}.AAAAAAAAAAAAAAAAAAAAAAAAA.{}", parts[0], parts[2]);
        let result = verify_token(&tampered);
        assert!(result.is_err(), "tampered token must be rejected");
    }
}
