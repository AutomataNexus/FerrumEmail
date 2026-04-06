//! Authentication against the Ferrum Mailbox API.
//! Handles login, session persistence, and API communication.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const API_BASE: &str = "https://ferrum-mail.com/mailbox/api/v1";
const SESSION_FILE: &str = "ferrum-session.json";

#[derive(Serialize, Deserialize, Clone)]
pub struct Session {
    pub token: String,
    pub username: String,
    pub email: String,
    pub display_name: String,
}

#[derive(Deserialize)]
struct LoginResponse {
    token: String,
    username: String,
    email: String,
    display_name: String,
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

    // Validate token by hitting a protected endpoint
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(format!("{API_BASE}/preferences"))
        .header("Authorization", format!("Bearer {}", session.token))
        .send()
        .ok()?;

    if resp.status().is_success() {
        Some(session)
    } else {
        std::fs::remove_file(session_path()).ok();
        None
    }
}

fn save_session(session: &Session) {
    if let Ok(data) = serde_json::to_string_pretty(session) {
        std::fs::write(session_path(), data).ok();
    }
}

/// Login to Ferrum Mailbox API.
pub fn login(username: &str, password: &str) -> Result<Session, String> {
    let client = reqwest::blocking::Client::new();

    let resp = client
        .post(format!("{API_BASE}/auth/login"))
        .json(&serde_json::json!({
            "username": username,
            "password": password,
        }))
        .send()
        .map_err(|e| format!("Connection failed: {e}"))?;

    if !resp.status().is_success() {
        let body = resp.text().unwrap_or_default();
        return Err(format!("Login failed: {body}"));
    }

    let login_resp: LoginResponse = resp.json().map_err(|e| format!("Parse error: {e}"))?;

    let session = Session {
        token: login_resp.token,
        username: login_resp.username.clone(),
        email: login_resp.email,
        display_name: login_resp.display_name,
    };

    save_session(&session);
    Ok(session)
}

/// Clear saved session (logout).
pub fn logout() {
    std::fs::remove_file(session_path()).ok();
}

// ── Mailbox API client ──

pub struct MailboxClient {
    token: String,
    client: reqwest::blocking::Client,
}

impl MailboxClient {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
            client: reqwest::blocking::Client::new(),
        }
    }

    fn get(&self, path: &str) -> Result<serde_json::Value, String> {
        let resp = self.client
            .get(format!("{API_BASE}{path}"))
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()));
        }
        resp.json().map_err(|e| e.to_string())
    }

    fn post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value, String> {
        let resp = self.client
            .post(format!("{API_BASE}{path}"))
            .header("Authorization", format!("Bearer {}", self.token))
            .json(body)
            .timeout(std::time::Duration::from_secs(60))
            .send()
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(body);
        }
        resp.json().map_err(|e| e.to_string())
    }

    pub fn list_folders(&self) -> Result<Vec<serde_json::Value>, String> {
        let v = self.get("/folders/")?;
        v.as_array().cloned().ok_or_else(|| "bad response".into())
    }

    pub fn list_messages(&self, folder: &str) -> Result<Vec<serde_json::Value>, String> {
        let v = self.get(&format!("/folders/{folder}"))?;
        v.as_array().cloned().ok_or_else(|| "bad response".into())
    }

    pub fn get_message(&self, folder: &str, id: &str) -> Result<serde_json::Value, String> {
        self.get(&format!("/messages/{folder}/{id}"))
    }

    pub fn send_email(
        &self,
        to: &[String],
        subject: &str,
        html: &str,
        text: Option<&str>,
    ) -> Result<String, String> {
        let resp = self.post("/send", &serde_json::json!({
            "to": to,
            "subject": subject,
            "html": html,
            "text": text,
        }))?;
        Ok(resp["message_id"].as_str().unwrap_or("sent").to_string())
    }

    pub fn move_message(&self, folder: &str, id: &str, to_folder: &str) -> Result<(), String> {
        self.post(
            &format!("/messages/{folder}/{id}/move"),
            &serde_json::json!({"to_folder": to_folder}),
        )?;
        Ok(())
    }
}
