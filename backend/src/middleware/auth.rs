use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::sync::Arc;

use crate::{config::Config, models::Claims};

#[allow(dead_code)]
#[derive(Clone)]
pub struct AuthContext {
    pub user_id: uuid::Uuid,
    pub correo: String,
    pub rol: String,
}

pub async fn require_auth(
    State(config): State<Arc<Config>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());
    let validation = Validation::default();

    match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(token_data) => {
            let auth_ctx = AuthContext {
                user_id: token_data.claims.sub,
                correo: token_data.claims.correo,
                rol: token_data.claims.rol,
            };
            req.extensions_mut().insert(auth_ctx);
            Ok(next.run(req).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
