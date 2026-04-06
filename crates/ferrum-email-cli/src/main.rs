//! ferrum — CLI for Ferrum Mail
//!
//! Commands: login, inbox, read, send, compose, folders, logout

use std::path::PathBuf;

const API_BASE: &str = "https://ferrum-mail.com/mailbox/api/v1";
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
        "inbox" | "ls" => cmd_inbox(&args),
        "folders" => cmd_folders(),
        "read" => cmd_read(&args),
        "send" => cmd_send(&args),
        "whoami" => cmd_whoami(),
        "help" | "--help" | "-h" => cmd_help(),
        _ => {
            eprintln!("Unknown command: {cmd}");
            cmd_help();
            std::process::exit(1);
        }
    }
}

fn cmd_help() {
    println!("Ferrum Mail CLI");
    println!();
    println!("USAGE: ferrum <command> [options]");
    println!();
    println!("COMMANDS:");
    println!("  login <username> <password>   Sign in to Ferrum Mail");
    println!("  logout                        Clear saved session");
    println!("  whoami                        Show current user info");
    println!("  inbox [folder]                List messages (default: inbox)");
    println!("  folders                       List all folders with counts");
    println!("  read <folder> <id>            Read a message");
    println!("  send <to> <subject> <body>    Send an email");
    println!("  help                          Show this help");
}

fn cmd_login(args: &[String]) {
    if args.len() < 4 {
        eprintln!("Usage: ferrum login <username> <password>");
        std::process::exit(1);
    }
    let username = &args[2];
    let password = &args[3];

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(format!("{API_BASE}/auth/login"))
        .json(&serde_json::json!({"username": username, "password": password}))
        .send();

    match resp {
        Ok(r) if r.status().is_success() => {
            let body: serde_json::Value = r.json().unwrap_or_default();
            let _ = std::fs::write(
                session_path(),
                serde_json::to_string_pretty(&body).unwrap_or_default(),
            );
            println!("Logged in as {}@ferrum-mail.com", body["username"].as_str().unwrap_or(username));
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
    match api_get("/preferences", &token) {
        Ok(prefs) => {
            println!("Username:  {}", prefs["username"].as_str().unwrap_or("?"));
            println!("Email:     {}", prefs["email"].as_str().unwrap_or("?"));
            println!("Name:      {}", prefs["display_name"].as_str().unwrap_or("?"));
            println!("Plan:      {}", prefs["plan"].as_str().unwrap_or("Free"));
            let used = prefs["storage_used"].as_u64().unwrap_or(0);
            let limit = prefs["storage_limit"].as_u64().unwrap_or(0);
            println!("Storage:   {} / {}", fmt_bytes(used), fmt_bytes(limit));
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

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
        eprintln!("  Multiple recipients: ferrum send \"a@b.com,c@d.com\" \"Subject\" \"Body\"");
        std::process::exit(1);
    }
    let to: Vec<String> = args[2].split(',').map(|s| s.trim().to_string()).collect();
    let subject = &args[3];
    let body = &args[4];
    let token = require_token();

    let html = format!(
        "<div style=\"font-family:-apple-system,sans-serif;color:#2D2A26;font-size:15px;line-height:1.6\">{}</div>",
        body.replace('\n', "<br>")
    );

    match api_post("/send", &token, &serde_json::json!({
        "to": to,
        "subject": subject,
        "html": html,
        "text": body,
    })) {
        Ok(resp) => {
            let mid = resp["message_id"].as_str().unwrap_or("sent");
            println!("Sent to {} (ID: {mid})", to.join(", "));
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
