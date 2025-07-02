mod authentication;
mod jsony;
mod server;

use server::start_server_interactive;
use jsony::{add_user_to_json, remove_user_from_json, list_users_from_json};
use std::io::{self, Write};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("axum=debug"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    loop {
        if let Err(e) = menu().await {
            eprintln!("Error in menu: {e}");
        }
    }
}

async fn menu() -> io::Result<()> {
    loop {
        println!("\nUser Management Menu:");
        println!("1. Add user");
        println!("2. Remove user");
        println!("3. List users");
        println!("4. Start server");
        println!("5. Exit");
        print!("Enter your choice: ");
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        match choice.trim() {
            "1" => {
                let (username, api_key) = prompt_user_and_key()?;
                add_user_to_json("users.json", &username, &api_key)?;
                println!("User '{}' added.", username);
            }
            "2" => {
                print!("Enter username to remove: ");
                io::stdout().flush()?;
                let mut username = String::new();
                io::stdin().read_line(&mut username)?;
                remove_user_from_json("users.json", username.trim())?;
                println!("User '{}' removed (if existed).", username.trim());
            }
            "3" => {
                list_users_from_json("users.json")?;
            }
            "4" => {
                if let Err(e) = start_server_interactive().await {
                    eprintln!("Server error: {e}");
                }
                break;
            }
            "5" => {
                println!("Exiting.");
                std::process::exit(0);
            }
            _ => println!("Invalid choice. Please try again."),
        }
    }
    Ok(())
}

fn prompt_user_and_key() -> io::Result<(String, String)> {
    print!("Enter username: ");
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;

    print!("Enter API key: ");
    io::stdout().flush()?;
    let mut api_key = String::new();
    io::stdin().read_line(&mut api_key)?;

    Ok((username.trim().to_string(), api_key.trim().to_string()))
}
