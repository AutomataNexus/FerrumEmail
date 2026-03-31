//! Authentication against the Ferrum Mail SaaS API.
//! Handles login, session persistence, and SMTP credential retrieval.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const API_BASE: &str = "https://ferrum-mail.com/v1";
const SESSION_FILE: &str = "ferrum-session.json";

#[derive(Serialize, Deserialize, Clone)]
pub struct Session {
    pub token: String,
    pub user_id: String,
    pub email: String,
    pub api_key: Option<String>,
}

#[derive(Deserialize)]
struct LoginResponse {
    token: String,
    user_id: String,
    email: String,
}

#[derive(Deserialize)]
struct KeyResponse {
    key: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct KeyListItem {
    prefix: String,
}

fn session_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ferrum-mail");
    std::fs::create_dir_all(&config_dir).ok();
    config_dir.join(SESSION_FILE)
}

/// Try to load a saved session from disk.
pub fn load_session() -> Option<Session> {
    let data = std::fs::read_to_string(session_path()).ok()?;
    let session: Session = serde_json::from_str(&data).ok()?;

    // Validate token is still valid by hitting the API
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(format!("{API_BASE}/dashboard"))
        .header("Authorization", format!("Bearer {}", session.token))
        .send()
        .ok()?;

    if resp.status().is_success() {
        Some(session)
    } else {
        // Token expired, clean up
        std::fs::remove_file(session_path()).ok();
        None
    }
}

/// Save session to disk.
fn save_session(session: &Session) {
    if let Ok(data) = serde_json::to_string_pretty(session) {
        std::fs::write(session_path(), data).ok();
    }
}

/// Login to Ferrum Mail SaaS and return a session.
pub fn login(email: &str, password: &str) -> Result<Session, String> {
    let client = reqwest::blocking::Client::new();

    let resp = client
        .post(format!("{API_BASE}/auth/login"))
        .json(&serde_json::json!({
            "email": email,
            "password": password,
        }))
        .send()
        .map_err(|e| format!("Connection failed: {e}"))?;

    if !resp.status().is_success() {
        let body = resp.text().unwrap_or_default();
        return Err(format!("Login failed: {body}"));
    }

    let login_resp: LoginResponse = resp.json().map_err(|e| format!("Parse error: {e}"))?;

    // Check if user already has an API key
    let api_key = get_or_create_api_key(&login_resp.token);

    let session = Session {
        token: login_resp.token,
        user_id: login_resp.user_id,
        email: login_resp.email,
        api_key,
    };

    save_session(&session);
    Ok(session)
}

/// Get existing API key or create one for the TUI.
fn get_or_create_api_key(token: &str) -> Option<String> {
    let client = reqwest::blocking::Client::new();

    // List existing keys
    let resp = client
        .get(format!("{API_BASE}/keys"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .ok()?;

    if resp.status().is_success() {
        let keys: Vec<KeyListItem> = resp.json().ok()?;
        // If user has keys, they already know their key — don't create another
        if !keys.is_empty() {
            return None; // User will use their existing key
        }
    }

    // Create a new key for the TUI
    let resp = client
        .post(format!("{API_BASE}/keys"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({"name": "Ferrum TUI"}))
        .send()
        .ok()?;

    if resp.status().is_success() {
        let key_resp: KeyResponse = resp.json().ok()?;
        Some(key_resp.key)
    } else {
        None
    }
}

/// Clear saved session (logout).
#[allow(dead_code)]
pub fn logout() {
    std::fs::remove_file(session_path()).ok();
}

/// Get SMTP configuration for sending via Ferrum Mail.
pub fn smtp_config_for_session(session: &Session) -> (String, u16, String, String) {
    // Use the API key as SMTP credentials against the Ferrum relay
    let smtp_host = "ferrum-mail.com".to_string();
    let smtp_port = 587u16;
    let smtp_user = session
        .api_key
        .clone()
        .unwrap_or_else(|| session.email.clone());
    let smtp_pass = session.token.clone();
    (smtp_host, smtp_port, smtp_user, smtp_pass)
}
