use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::{Request, StatusCode, header::AUTHORIZATION, request::Parts},
    middleware::Next,
    response::Response,
};
use axum::body::Body;
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use std::{
    collections::HashMap,
    fs::File,
    sync::Arc,
};
use tracing::{info, error};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub username: String,
    pub api_key: String,
}

#[derive(Clone)]
pub struct AppState {
    pub users: Arc<HashMap<String, String>>, // api_key -> username
}

pub fn load_users(path: &str) -> Result<Vec<User>, std::io::Error> {
    let file = File::open(path)?;
    let users: Vec<User> = from_reader(file)?;
    Ok(users)
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract and clone the Authorization header early
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);

    let auth_header = match auth_header {
        Some(h) => h,
        None => {
            error!("Missing Authorization header");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let token = match auth_header.strip_prefix("Bearer ").map(str::trim) {
        Some(t) => t,
        None => {
            error!("Malformed Authorization header");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let mut parts = token.splitn(2, ':');
    let username = parts.next().unwrap_or("");
    let api_key = parts.next().unwrap_or("");

    if username.is_empty() || api_key.is_empty() {
        error!("Malformed Authorization token, expected 'username:api_key'");
        return Err(StatusCode::UNAUTHORIZED);
    }

    if let Some(expected_key) = state.users.get(username) {
        if expected_key == api_key {
            info!("Authenticated user: {}", username);
            req.extensions_mut().insert(username.to_string());
            Ok(next.run(req).await)
        } else {
            error!("Invalid API key for user '{}'", username);
            Err(StatusCode::UNAUTHORIZED)
        }
    } else {
        error!("Unknown username '{}'", username);
        Err(StatusCode::UNAUTHORIZED)
    }
}


// Extractor to get authenticated username from request extensions
pub struct AuthenticatedUser(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<String>()
            .cloned()
            .map(AuthenticatedUser)
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}
