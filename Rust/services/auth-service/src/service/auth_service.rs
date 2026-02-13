use anyhow::{Result, anyhow};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, Header, EncodingKey};
use chrono::{Utc, Duration};
use serde::{Serialize, Deserialize};

use crate::domain::UserPublic;
use crate::repo::UserRepository;

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    user_id: i32,
    exp: i64,
}

#[derive(Clone)]
pub struct AuthService {
    user_repo: UserRepository,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(user_repo: UserRepository) -> Self {
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "RushTech@2025xAjxh".to_string());

        Self {
            user_repo,
            jwt_secret,
        }
    }

    pub async fn register(&self, name: &str, email: &str, password: &str) -> Result<(String, UserPublic)> {
        // Check if email already exists
        if self.user_repo.email_exists(email).await? {
            return Err(anyhow!("Email already registered"));
        }

        // Hash password
        let hashed_password = hash(password, DEFAULT_COST)
            .map_err(|e| anyhow!("Failed to hash password: {}", e))?;

        // Create user
        let user_id = self.user_repo.create(name, email, &hashed_password).await?;

        // Generate token
        let token = self.generate_token(user_id, email)?;

        let user_public = UserPublic {
            id: user_id,
            name: name.to_string(),
            email: email.to_string(),
        };

        Ok((token, user_public))
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<(String, UserPublic)> {
        // Find user by email
        let user = self.user_repo.find_by_email(email).await?
            .ok_or_else(|| anyhow!("Invalid credentials"))?;

        // Verify password
        let password_match = verify(password, &user.password)
            .map_err(|e| anyhow!("Password verification failed: {}", e))?;

        if !password_match {
            return Err(anyhow!("Invalid credentials"));
        }

        // Generate token
        let token = self.generate_token(user.id, &user.email)?;

        let user_public = UserPublic::from(user);

        Ok((token, user_public))
    }

    fn generate_token(&self, user_id: i32, email: &str) -> Result<String> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .ok_or_else(|| anyhow!("Invalid timestamp"))?
            .timestamp();

        let claims = Claims {
            sub: email.to_string(),
            user_id,
            exp: expiration,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| anyhow!("Token generation failed: {}", e))?;

        Ok(token)
    }
}
