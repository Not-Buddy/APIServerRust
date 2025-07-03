# APIServerRust

A secure, menu-driven RESTful API server in Rust using [Axum](https://docs.rs/axum), with user authentication based on a local JSON file.

## Features

- **Axum-based REST API**: Fast, async, and modular.
- **API Key Authentication**: Requires both username and API key, sent as `Authorization: Bearer :`.
- **User Management CLI**: Add, remove, and list users interactively before starting the server.
- **Protected Endpoint**: `/data` route returns dummy JSON data only for authenticated users.
- **Graceful Server Control**: Start and stop the server interactively, returning to the menu after shutdown.
- **Logging**: Uses `tracing` for structured logs.
- **Idiomatic Rust**: Modular code, robust error handling, and clear separation of concerns.

## Project Structure

```
src/
├── main.rs            # CLI menu and entry point
├── authentication.rs  # Auth logic and user model
├── jsony.rs           # User JSON file management
└── server.rs          # Server startup and shutdown logic
users.json             # User credentials (username + API key)
Cargo.toml
```

## Usage

### 1. User Management

On running `cargo run`, you'll see a menu:

```
User Management Menu:
1. Add user
2. Remove user
3. Start server
4. Exit
```

- **Add user**: Enter a username and API key. The user is added to `users.json`.
- **Remove user**: Enter a username to remove.
- **Start server**: Launches the API server. Press Enter to stop and return to the menu.

### 2. API Authentication

- The API expects the header:
  ```
  Authorization: Bearer :
  ```
- Example (PowerShell):
  ```powershell
  Invoke-WebRequest -Uri "http://127.0.0.1:3000/data" `
    -Headers @{ Authorization = "Bearer alice:some-secret-key-123" }
  ```

### 3. Example `users.json`

```json
[
  { "username": "alice", "api_key": "some-secret-key-123" },
  { "username": "bob", "api_key": "another-key-456" }
]
```

## Dependencies

- `axum`
- `tokio`
- `serde`, `serde_json`
- `tower-http`
- `tracing`, `tracing-subscriber`
- `axum-server`

## Security Notes

- The API only authenticates requests with a valid username and matching API key.
- The `users.json` file is loaded once at startup and kept in memory.

**This project is a template for secure, menu-driven API servers in Rust with simple, local user management.**