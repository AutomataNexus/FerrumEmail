//! Authentication against the Ferrum Mail SaaS API.
//! Handles login, session persistence, and API key retrieval.

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

    // Validate token by hitting a protected endpoint
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(format!("{API_BASE}/dashboard"))
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

/// Login to Ferrum Mail SaaS.
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

    // Get or create an API key for sending
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
        if !keys.is_empty() {
            return None; // User has keys, they'll use their existing one
        }
    }

    // Create a new key
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

/// Get SMTP config for sending via the SaaS relay.
#[allow(dead_code)]
pub fn smtp_config_for_session(session: &Session) -> (String, u16, String, String) {
    let smtp_host = "ferrum-mail.com".to_string();
    let smtp_port = 587u16;
    let smtp_user = session
        .api_key
        .clone()
        .unwrap_or_else(|| session.email.clone());
    let smtp_pass = session.token.clone();
    (smtp_host, smtp_port, smtp_user, smtp_pass)
}

// ── SaaS API Client ──

pub struct SaasClient {
    token: String,
    client: reqwest::blocking::Client,
}

impl SaasClient {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
            client: reqwest::blocking::Client::new(),
        }
    }

    fn get(&self, path: &str) -> Result<serde_json::Value, String> {
        let resp = self
            .client
            .get(format!("{API_BASE}{path}"))
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()));
        }
        resp.json().map_err(|e| e.to_string())
    }

    #[allow(dead_code)]
    fn post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value, String> {
        let resp = self
            .client
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

    pub fn dashboard(&self) -> Result<serde_json::Value, String> {
        self.get("/dashboard")
    }

    pub fn send_history(&self) -> Result<Vec<serde_json::Value>, String> {
        let v = self.get("/sends")?;
        v.as_array().cloned().ok_or_else(|| "bad response".into())
    }

    pub fn list_keys(&self) -> Result<Vec<serde_json::Value>, String> {
        let v = self.get("/keys")?;
        v.as_array().cloned().ok_or_else(|| "bad response".into())
    }

    /// Send email via the SaaS API.
    pub fn send_email(
        &self,
        to: &str,
        subject: &str,
        html: &str,
        text: Option<&str>,
        api_key: Option<&str>,
    ) -> Result<String, String> {
        let client = reqwest::blocking::Client::new();
        let auth = if let Some(key) = api_key {
            format!("Bearer {key}")
        } else {
            format!("Bearer {}", self.token)
        };

        let resp = client
            .post(format!("{API_BASE}/emails"))
            .header("Authorization", auth)
            .json(&serde_json::json!({
                "to": to,
                "subject": subject,
                "html": html,
                "text": text,
            }))
            .timeout(std::time::Duration::from_secs(60))
            .send()
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(resp.text().unwrap_or_default());
        }

        let body: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
        Ok(body["message_id"].as_str().unwrap_or("sent").to_string())
    }
}
