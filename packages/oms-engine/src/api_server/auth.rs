//! JWT Authentication Middleware

use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api_server::error::ApiError;
use crate::api_server::state::AppState;
use crate::api_server::types::{LoginRequest, LoginResponse};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub exp: usize,
    pub iat: usize,
}

/// JWT authentication middleware
pub async fn jwt_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Extract token from Authorization header
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let token = match auth_header {
        Some(value) if value.starts_with("Bearer ") => &value[7..],
        _ => {
            return Err(ApiError::Unauthorized {
                message: "Missing or invalid Authorization header".to_string(),
            });
        }
    };

    // Validate token
    let claims = validate_token(token, &state.jwt_secret)?;

    // Add claims to request extensions for use in handlers
    req.extensions_mut().insert(claims);

    // Call next handler
    Ok(next.run(req).await)
}

/// Validate JWT token and return claims
pub fn validate_token(token: &str, secret: &str) -> Result<Claims, ApiError> {
    let validation = Validation::default();

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    )
    .map_err(|e| ApiError::InvalidToken {
        message: format!("Token validation failed: {}", e),
    })?;

    Ok(token_data.claims)
}

/// Generate JWT token for user
pub fn generate_token(user_id: &str, secret: &str) -> String {
    let now = Utc::now();
    let exp = now + Duration::hours(24);

    let claims = Claims {
        sub: user_id.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .expect("JWT encoding should not fail with valid claims")
}

/// Login handler - generates JWT token
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    // For now, accept any username/password
    // In production, verify against user database
    if req.username.is_empty() || req.password.is_empty() {
        return Err(ApiError::ValidationError {
            field: "credentials".to_string(),
            message: "Username and password are required".to_string(),
        });
    }

    // Generate token
    let token = generate_token(&req.username, &state.jwt_secret);

    Ok(Json(LoginResponse {
        token,
        expires_in: 86400, // 24 hours
    }))
}

/// Extract user ID from request extensions (set by jwt_middleware)
pub fn get_user_id_from_request(req: &Request) -> Option<String> {
    req.extensions()
        .get::<Claims>()
        .map(|claims| claims.sub.clone())
}
