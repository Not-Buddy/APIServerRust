use axum::{
    async_trait,
    extract::{State, FromRequestParts},
    http::{Request, StatusCode, header::AUTHORIZATION, request::Parts},
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};

use axum::body::Body;
use serde::{Deserialize};
use serde_json::from_reader;
use std::{
    collections::HashMap,
    fs::File,
    net::SocketAddr,
    sync::Arc,
};
use tower_http::trace::TraceLayer;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Deserialize, Clone)]
struct User {
    username: String,
    api_key: String,
}

#[derive(Clone)]
struct AppState {
    users: Arc<HashMap<String, String>>, // api_key -> username
}

#[tokio::main]
async fn main() {
    // Initialize logging/tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("axum=debug"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load users.json at startup
    let users = load_users("users.json").expect("Failed to load users.json");
    let users_map = users.into_iter()
        .map(|u| (u.api_key, u.username))
        .collect::<HashMap<_, _>>();

    let state = AppState {
        users: Arc::new(users_map),
    };

    // Build the application
    let app = Router::new()
        .route("/data", get(protected_data))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state.clone())
        .layer(TraceLayer::new_for_http());

    // Start the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Listening on {}", addr);

    axum_server::bind(addr)
    .serve(app.into_make_service())
    .await
    .unwrap();

}

// Read users.json into Vec<User>
fn load_users(path: &str) -> Result<Vec<User>, std::io::Error> {
    let file = File::open(path)?;
    let users: Vec<User> = from_reader(file)?;
    Ok(users)
}

async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract Authorization header
    let auth_header = req.headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let Some(auth_header) = auth_header else {
        error!("Missing Authorization header");
        return Err(StatusCode::UNAUTHORIZED);
    };

    // Expect header format: "Bearer <api_key>"
    let api_key = auth_header.strip_prefix("Bearer ").map(str::trim);

    let Some(api_key) = api_key else {
        error!("Malformed Authorization header");
        return Err(StatusCode::UNAUTHORIZED);
    };

    // Validate API key
    if let Some(username) = state.users.get(api_key) {
        info!("Authenticated user: {}", username);
        // Optionally, insert user info into request extensions for handler access
        req.extensions_mut().insert(username.clone());
        Ok(next.run(req).await)
    } else {
        error!("Invalid API key");
        Err(StatusCode::UNAUTHORIZED)
    }
}

// Handler for GET /data
async fn protected_data(
    State(_state): State<AppState>,
    username: AuthenticatedUser,
) -> impl IntoResponse {
    Json(serde_json::json!({
        "message": format!("Hello, {}! Here is your protected data.", username.0),
        "data": [1, 2, 3, 4]
    }))
}

// Extractor to get authenticated username from request extensions
struct AuthenticatedUser(String);

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
