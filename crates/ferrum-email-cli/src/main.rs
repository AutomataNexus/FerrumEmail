//! ferrum — CLI for Ferrum Mail
//!
//! Commands: login, inbox, read, send, compose, folders, logout

use std::path::PathBuf;

const API_BASE: &str = "https://ferrum-mail.com/v1";
const SESSION_FILE: &str = "ferrum-session.json";

fn session_path() -> PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ferrum-mail");
    std::fs::create_dir_all(&dir).ok();
    dir.join(SESSION_FILE)
}

fn load_token() -> Option<String> {
    let data = std::fs::read_to_string(session_path()).ok()?;
    let v: serde_json::Value = serde_json::from_str(&data).ok()?;
    v["token"].as_str().map(|s| s.to_string())
}

fn require_token() -> String {
    load_token().unwrap_or_else(|| {
        eprintln!("Not logged in. Run: ferrum login");
        std::process::exit(1);
    })
}

fn api_get(path: &str, token: &str) -> Result<serde_json::Value, String> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(format!("{API_BASE}{path}"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}: {}", resp.status(), resp.text().unwrap_or_default()));
    }
    resp.json().map_err(|e| e.to_string())
}

fn api_post(path: &str, token: &str, body: &serde_json::Value) -> Result<serde_json::Value, String> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(format!("{API_BASE}{path}"))
        .header("Authorization", format!("Bearer {token}"))
        .json(body)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}: {}", resp.status(), resp.text().unwrap_or_default()));
    }
    resp.json().map_err(|e| e.to_string())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match cmd {
        "login" => cmd_login(&args),
        "logout" => cmd_logout(),
        "send" => cmd_send(&args),
        "whoami" => cmd_whoami(),
        "keys" => cmd_keys(),
        "history" => cmd_history(),
        "help" | "--help" | "-h" => cmd_help(),
        _ => {
            eprintln!("Unknown command: {cmd}");
            cmd_help();
            std::process::exit(1);
        }
    }
}

fn cmd_help() {
    println!("Ferrum Email CLI — send transactional emails from the command line");
    println!();
    println!("USAGE: ferrum <command> [options]");
    println!();
    println!("COMMANDS:");
    println!("  login <email> <password>   Sign in to ferrum-mail.com");
    println!("  logout                     Clear saved session");
    println!("  whoami                     Show account info + plan");
    println!("  send <to> <subject> <body> Send a transactional email");
    println!("  keys                       List your API keys");
    println!("  history                    Show send history");
    println!("  help                       Show this help");
    println!();
    println!("EXAMPLES:");
    println!("  ferrum login me@example.com mypassword");
    println!("  ferrum send user@example.com \"Welcome\" \"Hello, welcome to our app!\"");
    println!("  ferrum keys");
}

fn cmd_login(args: &[String]) {
    if args.len() < 4 {
        eprintln!("Usage: ferrum login <email> <password>");
        std::process::exit(1);
    }
    let email = &args[2];
    let password = &args[3];

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(format!("{API_BASE}/auth/login"))
        .json(&serde_json::json!({"email": email, "password": password}))
        .send();

    match resp {
        Ok(r) if r.status().is_success() => {
            let body: serde_json::Value = r.json().unwrap_or_default();
            let _ = std::fs::write(
                session_path(),
                serde_json::to_string_pretty(&body).unwrap_or_default(),
            );
            println!("Logged in as {}", body["email"].as_str().unwrap_or(email));
        }
        Ok(r) => {
            eprintln!("Login failed: {}", r.text().unwrap_or_default());
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Connection failed: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_logout() {
    std::fs::remove_file(session_path()).ok();
    println!("Logged out.");
}

fn cmd_whoami() {
    let token = require_token();
    match api_get("/dashboard", &token) {
        Ok(dash) => {
            println!("Email:       {}", dash["email"].as_str().unwrap_or("?"));
            println!("Plan:        {}", dash["plan"].as_str().unwrap_or("Free"));
            println!("Sends:       {} total", dash["total_sends"].as_u64().unwrap_or(0));
            println!("Today:       {} sent", dash["sends_today"].as_u64().unwrap_or(0));
            if let Some(quota) = dash["monthly_quota"].as_str().or(dash["quota"].as_str()) {
                println!("Quota:       {quota}/mo");
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_keys() {
    let token = require_token();
    match api_get("/keys", &token) {
        Ok(v) => {
            if let Some(keys) = v.as_array() {
                if keys.is_empty() {
                    println!("No API keys. Create one at ferrum-mail.com/dashboard/keys");
                    return;
                }
                println!("{:<20} {:<12} {}", "PREFIX", "STATUS", "CREATED");
                println!("{}", "-".repeat(50));
                for k in keys {
                    println!(
                        "{:<20} {:<12} {}",
                        k["prefix"].as_str().unwrap_or("?"),
                        if k["revoked"].as_bool().unwrap_or(false) { "revoked" } else { "active" },
                        k["created_at"].as_str().unwrap_or("?"),
                    );
                }
            }
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}

fn cmd_history() {
    let token = require_token();
    match api_get("/sends", &token) {
        Ok(v) => {
            if let Some(sends) = v.as_array() {
                if sends.is_empty() {
                    println!("No emails sent yet.");
                    return;
                }
                println!("{:<12} {:<30} {:<10} {}", "DATE", "TO", "STATUS", "ID");
                println!("{}", "-".repeat(70));
                for s in sends.iter().rev().take(20) {
                    let date = s["sent_at"].as_str().unwrap_or("");
                    let short = if date.len() > 10 { &date[..10] } else { date };
                    println!(
                        "{:<12} {:<30} {:<10} {}",
                        short,
                        s["to"].as_str().unwrap_or("?"),
                        s["status"].as_str().unwrap_or("?"),
                        s["message_id"].as_str().unwrap_or(""),
                    );
                }
                println!("\n{} total sends", sends.len());
            }
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}

// Removed mailbox commands (inbox, folders, read) — those are for Ferrum Mailbox clients

#[allow(dead_code)]
fn cmd_folders() {
    let token = require_token();
    match api_get("/folders/", &token) {
        Ok(v) => {
            if let Some(folders) = v.as_array() {
                println!("{:<12} {:>6} {:>6}", "FOLDER", "TOTAL", "UNREAD");
                println!("{}", "-".repeat(26));
                for f in folders {
                    println!(
                        "{:<12} {:>6} {:>6}",
                        f["name"].as_str().unwrap_or("?"),
                        f["total"].as_u64().unwrap_or(0),
                        f["unread"].as_u64().unwrap_or(0),
                    );
                }
            }
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}

fn cmd_inbox(args: &[String]) {
    let folder = args.get(2).map(|s| s.as_str()).unwrap_or("inbox");
    let token = require_token();
    match api_get(&format!("/folders/{folder}"), &token) {
        Ok(v) => {
            if let Some(msgs) = v.as_array() {
                if msgs.is_empty() {
                    println!("No messages in {folder}.");
                    return;
                }
                for msg in msgs {
                    let unread = if msg["read"].as_bool().unwrap_or(false) { " " } else { "*" };
                    let from = msg["from_display"].as_str()
                        .or_else(|| msg["from"].as_str())
                        .unwrap_or("?");
                    let subject = msg["subject"].as_str().unwrap_or("(no subject)");
                    let id = msg["id"].as_str().unwrap_or("");
                    let date = msg["received_at"].as_str().unwrap_or("");
                    let short_date = if date.len() > 10 { &date[..10] } else { date };
                    println!("{unread} {short_date}  {:<20}  {:<40}  {}", from, subject, &id[..8.min(id.len())]);
                }
                println!("\n{} messages. Use: ferrum read {folder} <id>", msgs.len());
            }
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}

fn cmd_read(args: &[String]) {
    if args.len() < 4 {
        eprintln!("Usage: ferrum read <folder> <id>");
        std::process::exit(1);
    }
    let folder = &args[2];
    let id = &args[3];
    let token = require_token();

    match api_get(&format!("/messages/{folder}/{id}"), &token) {
        Ok(detail) => {
            let meta = &detail["meta"];
            println!("Subject: {}", meta["subject"].as_str().unwrap_or("?"));
            println!("From:    {}", meta["from"].as_str().unwrap_or("?"));
            println!("To:      {}", meta["to"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
                .unwrap_or_default());
            println!("Date:    {}", meta["date"].as_str().unwrap_or("?"));
            println!("{}", "-".repeat(60));

            let body = detail["text_body"].as_str()
                .or_else(|| detail["html_body"].as_str())
                .unwrap_or("(empty)");

            // Strip HTML tags for display
            if body.contains('<') {
                let stripped: String = {
                    let mut clean = String::new();
                    let mut in_tag = false;
                    for ch in body.replace("<br>", "\n").replace("</p>", "\n\n").chars() {
                        match ch {
                            '<' => in_tag = true,
                            '>' => in_tag = false,
                            _ if !in_tag => clean.push(ch),
                            _ => {}
                        }
                    }
                    clean
                };
                println!("{}", stripped.trim());
            } else {
                println!("{body}");
            }
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}

fn cmd_send(args: &[String]) {
    if args.len() < 5 {
        eprintln!("Usage: ferrum send <to> <subject> <body>");
        eprintln!("  Example: ferrum send user@example.com \"Welcome\" \"Hello, welcome to our app!\"");
        std::process::exit(1);
    }
    let to = &args[2];
    let subject = &args[3];
    let body = &args[4];
    let token = require_token();

    let html = format!(
        "<div style=\"font-family:-apple-system,sans-serif;color:#2D2A26;font-size:15px;line-height:1.6\">{}</div>",
        body.replace('\n', "<br>")
    );

    match api_post("/emails", &token, &serde_json::json!({
        "to": to,
        "subject": subject,
        "html": html,
        "text": body,
    })) {
        Ok(resp) => {
            let mid = resp["message_id"].as_str().unwrap_or("sent");
            println!("Sent to {to} (ID: {mid})");
        }
        Err(e) => {
            eprintln!("Send failed: {e}");
            std::process::exit(1);
        }
    }
}

fn fmt_bytes(b: u64) -> String {
    if b < 1024 { return format!("{b}B"); }
    if b < 1048576 { return format!("{:.1}KB", b as f64 / 1024.0); }
    if b < 1073741824 { return format!("{:.1}MB", b as f64 / 1048576.0); }
    format!("{:.2}GB", b as f64 / 1073741824.0)
}
