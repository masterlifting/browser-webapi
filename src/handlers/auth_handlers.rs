//! Authentication handlers
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::{
    auth,
    error::AppError,
    models::ApiResponse,
};

/// Represents a login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Response containing the JWT token
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub expires_at: String,
}

/// Generates a JWT token for a user
pub async fn generate_token(req: web::Json<LoginRequest>) -> Result<impl Responder, AppError> {
    // In a real application, this would validate the username and password against a database
    // For this example, we'll use a simplified approach
    
    // Determine role based on username (in real applications this would come from a database)
    let role = match req.username.as_str() {
        "admin" => "admin",
        _ => "user",
    };
    
    // Generate the JWT token
    let token = auth::generate_token(&req.username, role)?;
    
    // Calculate expiry time (24 hours from now)
    use chrono::{Utc, Duration};
    let expires_at = (Utc::now() + Duration::hours(24)).to_rfc3339();
    
    // Return the token in the response
    let response = TokenResponse {
        token,
        expires_at,
    };
    
    Ok(HttpResponse::Ok().json(ApiResponse::new(response)))
}

/// Validates a JWT token
pub async fn validate_token(req: web::HttpRequest) -> Result<impl Responder, AppError> {
    // Extract the bearer token from the Authorization header
    let auth_header = req.headers().get("Authorization");
    let auth_header = match auth_header {
        Some(header) => header.to_str().unwrap_or(""),
        None => return Err(AppError::AuthError("Missing Authorization header".to_string())),
    };
    
    // Check if it starts with "Bearer "
    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::AuthError("Invalid Authorization header format".to_string()));
    }
    
    // Extract the token
    let token = &auth_header[7..]; // Skip "Bearer "
    
    // Validate the token
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret_key".to_string());
    
    use jsonwebtoken::{decode, DecodingKey, Validation};
    
    let token_data = match decode::<auth::Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => data,
        Err(_) => return Err(AppError::AuthError("Invalid token".to_string())),
    };
    
    // Return the claims in the response
    let response = serde_json::json!({
        "valid": true,
        "sub": token_data.claims.sub,
        "role": token_data.claims.role,
        "exp": token_data.claims.exp,
    });
    
    Ok(HttpResponse::Ok().json(ApiResponse::new(response)))
}
