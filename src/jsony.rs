use crate::authentication::User;
use serde_json::{from_reader, to_writer_pretty};
use std::{
    fs::{File, OpenOptions},
    io::{self, BufReader},
};

pub fn add_user_to_json(path: &str, username: &str, api_key: &str) -> io::Result<()> {
    let mut users = load_users_from_json(path)?;
    if users.iter().any(|u| u.username == username) {
        eprintln!("User '{}' already exists.", username);
        return Ok(());
    }
    users.push(User {
        username: username.to_string(),
        api_key: api_key.to_string(),
    });
    save_users_to_json(path, &users)
}

pub fn remove_user_from_json(path: &str, username: &str) -> io::Result<()> {
    let mut users = load_users_from_json(path)?;
    let original_len = users.len();
    users.retain(|u| u.username != username);
    if users.len() == original_len {
        eprintln!("User '{}' not found.", username);
    }
    save_users_to_json(path, &users)
}

pub fn list_users_from_json(path: &str) -> io::Result<()> {
    let users = load_users_from_json(path)?;
    if users.is_empty() {
        println!("No users found.");
    } else {
        println!("Current users:");
        for user in users {
            println!("Username: {}, API Key: {}", user.username, user.api_key);
        }
    }
    Ok(())
}

fn load_users_from_json(path: &str) -> io::Result<Vec<User>> {
    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        Ok(from_reader(reader).unwrap_or_else(|_| vec![]))
    } else {
        Ok(vec![])
    }
}

fn save_users_to_json(path: &str, users: &[User]) -> io::Result<()> {
    let file = OpenOptions::new().write(true).create(true).truncate(true).open(path)?;
    to_writer_pretty(file, users)?;
    Ok(())
}
