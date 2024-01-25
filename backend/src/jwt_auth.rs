use serde::{Deserialize, Serialize};
use axum::{
    extract::{Request, State},
    http::{StatusCode,header},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
    
};
use jsonwebtoken::{decode, DecodingKey, Validation};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub name: String,
    pub role: String,
    pub iat: usize,
    pub exp: usize,
}

pub async fn auth_jwt(
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    let token = req.headers()
    .get(header::AUTHORIZATION)
    .and_then(|auth_header| auth_header.to_str().ok())
    .and_then(|auth_value| {
        if auth_value.starts_with("Bearer ") {
            Some(auth_value[7..].to_owned())
        } else {
            None
        }
    });

    let token = token.ok_or_else(|| {
        return StatusCode::UNAUTHORIZED;
    })?;

    // FIXME: use secret
    let claims = decode::<TokenClaims>(
        &token,
        &DecodingKey::from_secret(b""),
        &Validation::default()
    ).map_err(|_| {
        return StatusCode::UNAUTHORIZED;
    })?.claims;

    if claims.role != "admin" {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(req).await)
}