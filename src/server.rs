// server.rs

use axum::{
    extract::State,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde_json::json;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
};
use tower_http::trace::TraceLayer;
use crate::authentication::{AppState, load_users, auth_middleware, AuthenticatedUser};
use axum_server::Handle;
use tokio::task;

pub async fn start_server_interactive() -> Result<(), Box<dyn std::error::Error>> {
    let users = load_users("users.json")?;
    let users_map = users.into_iter()
        .map(|u| (u.username, u.api_key))
        .collect::<HashMap<_, _>>();
    let state = AppState {
        users: Arc::new(users_map),
    };
    let app = Router::new()
        .route("/data", get(protected_data))
        .layer(axum::middleware::from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state.clone())
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server is running at http://127.0.0.1:3000");
    println!("Press Enter to stop the server and return to the menu...");

    let handle = Handle::new();
    let handle_clone = handle.clone();

    let server = axum_server::bind(addr)
        .handle(handle_clone)
        .serve(app.into_make_service());

    let server_handle = task::spawn(server);

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    handle.shutdown();
    server_handle.await??;

    println!("Server stopped. Returning to menu.");
    Ok(())
}

async fn protected_data(
    State(_state): State<AppState>,
    username: AuthenticatedUser,
) -> impl IntoResponse {
    Json(json!({
        "message": format!("Hello, {}! Here is your protected data.", username.0),
        "data": [1, 2, 3, 4]
    }))
}
