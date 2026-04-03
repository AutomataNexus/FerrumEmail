//! NexusRelay SMTP Proxy — HTTP-to-SMTP relay via authenticated submission.
//!
//! Runs on a machine with outbound port 587 access.
//! Accepts email relay requests over HTTP (Tailscale) and submits
//! through an authenticated SMTP relay (Office 365).

use axum::{extract::Json, http::StatusCode, routing::post, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const CMD_TIMEOUT: Duration = Duration::from_secs(30);

// SMTP submission relay config (loaded from env)
struct RelayConfig {
    smtp_host: String,
    smtp_port: u16,
    smtp_user: String,
    smtp_pass: String,
}

#[derive(Deserialize)]
struct RelayRequest {
    from: String,
    to: Vec<String>,
    mime_body: String,
}

#[derive(Serialize)]
struct RelayResponse {
    message_id: String,
    status: String,
}

#[derive(Serialize)]
struct RelayError {
    error: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = Arc::new(RelayConfig {
        smtp_host: std::env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.office365.com".into()),
        smtp_port: std::env::var("SMTP_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(587),
        smtp_user: std::env::var("SMTP_USER").expect("SMTP_USER required"),
        smtp_pass: std::env::var("SMTP_PASS").expect("SMTP_PASS required"),
    });

    tracing::info!(
        "NexusRelay using {}:{} as {}",
        config.smtp_host,
        config.smtp_port,
        config.smtp_user
    );

    let app = Router::new()
        .route("/relay", post(handle_relay))
        .route("/health", axum::routing::get(|| async { "ok" }))
        .layer(axum::extract::DefaultBodyLimit::max(26_214_400)) // 25MB
        .with_state(config);

    let addr = "0.0.0.0:9587";
    tracing::info!("NexusRelay listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_relay(
    axum::extract::State(config): axum::extract::State<Arc<RelayConfig>>,
    Json(req): Json<RelayRequest>,
) -> Result<Json<RelayResponse>, (StatusCode, Json<RelayError>)> {
    tracing::info!("Relay: {} -> {}", req.from, req.to.join(", "));

    match deliver_via_submission(&config, &req.from, &req.to, &req.mime_body).await {
        Ok(mid) => {
            tracing::info!("Delivered via {}", config.smtp_host);
            Ok(Json(RelayResponse {
                message_id: mid,
                status: "sent".into(),
            }))
        }
        Err(e) => {
            tracing::error!("Relay failed: {e}");
            Err((
                StatusCode::BAD_GATEWAY,
                Json(RelayError { error: e }),
            ))
        }
    }
}

async fn deliver_via_submission(
    config: &RelayConfig,
    from: &str,
    to: &[String],
    mime_body: &str,
) -> Result<String, String> {
    let addr = format!("{}:{}", config.smtp_host, config.smtp_port);
    let tcp = timeout(CONNECT_TIMEOUT, TcpStream::connect(&addr))
        .await
        .map_err(|_| format!("timeout connecting to {addr}"))?
        .map_err(|e| format!("connect to {addr}: {e}"))?;

    let (reader, mut writer) = tokio::io::split(tcp);
    let mut reader = BufReader::new(reader);

    // Greeting
    let greeting = read_resp(&mut reader).await?;
    if !greeting.starts_with('2') {
        return Err(format!("greeting: {greeting}"));
    }

    // EHLO
    send_cmd(&mut writer, "EHLO nexus-relay.ferrum-mail.com\r\n").await?;
    let ehlo = read_resp(&mut reader).await?;
    if !ehlo.starts_with('2') {
        return Err(format!("EHLO: {ehlo}"));
    }

    // STARTTLS
    send_cmd(&mut writer, "STARTTLS\r\n").await?;
    let tls_resp = read_resp(&mut reader).await?;
    if !tls_resp.starts_with('2') {
        return Err(format!("STARTTLS: {tls_resp}"));
    }

    // TLS upgrade
    let server_name = rustls::pki_types::ServerName::try_from(config.smtp_host.clone())
        .map_err(|e| format!("bad server name: {e}"))?;
    let root_store =
        rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let tls_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    let tcp = reader.into_inner().unsplit(writer);
    let connector = tokio_rustls::TlsConnector::from(Arc::new(tls_config));
    let tls_stream = connector
        .connect(server_name, tcp)
        .await
        .map_err(|e| format!("TLS: {e}"))?;

    let (tls_reader, mut tls_writer) = tokio::io::split(tls_stream);
    let mut tls_reader = BufReader::new(tls_reader);

    // EHLO again
    send_cmd(&mut tls_writer, "EHLO nexus-relay.ferrum-mail.com\r\n").await?;
    let _ = read_resp(&mut tls_reader).await?;

    // AUTH LOGIN
    send_cmd(&mut tls_writer, "AUTH LOGIN\r\n").await?;
    let auth1 = read_resp(&mut tls_reader).await?;
    if !auth1.starts_with('3') {
        return Err(format!("AUTH LOGIN: {auth1}"));
    }

    use base64::Engine;
    let user_b64 = base64::engine::general_purpose::STANDARD.encode(config.smtp_user.as_bytes());
    send_cmd(&mut tls_writer, &format!("{user_b64}\r\n")).await?;
    let auth2 = read_resp(&mut tls_reader).await?;
    if !auth2.starts_with('3') {
        return Err(format!("AUTH user: {auth2}"));
    }

    let pass_b64 = base64::engine::general_purpose::STANDARD.encode(config.smtp_pass.as_bytes());
    send_cmd(&mut tls_writer, &format!("{pass_b64}\r\n")).await?;
    let auth3 = read_resp(&mut tls_reader).await?;
    if !auth3.starts_with('2') {
        return Err(format!("AUTH pass: {auth3}"));
    }

    tracing::info!("Authenticated to {}", config.smtp_host);

    // MAIL FROM — use the authenticated account (O365 requires envelope matches auth)
    send_cmd(
        &mut tls_writer,
        &format!("MAIL FROM:<{}>\r\n", config.smtp_user),
    )
    .await?;
    let r = read_resp(&mut tls_reader).await?;
    if !r.starts_with('2') {
        return Err(format!("MAIL FROM: {r}"));
    }

    // RCPT TO
    for rcpt in to {
        send_cmd(&mut tls_writer, &format!("RCPT TO:<{rcpt}>\r\n")).await?;
        let r = read_resp(&mut tls_reader).await?;
        if !r.starts_with('2') {
            return Err(format!("RCPT TO <{rcpt}>: {r}"));
        }
    }

    // DATA
    send_cmd(&mut tls_writer, "DATA\r\n").await?;
    let r = read_resp(&mut tls_reader).await?;
    if !r.starts_with('3') {
        return Err(format!("DATA: {r}"));
    }

    // Rewrite From header to match authenticated account (O365 requires it)
    // Add Reply-To with original sender so replies go to the right place
    let rewritten = rewrite_from(mime_body, &config.smtp_user, from);
    tls_writer
        .write_all(rewritten.as_bytes())
        .await
        .map_err(|e| format!("write body: {e}"))?;
    send_cmd(&mut tls_writer, "\r\n.\r\n").await?;
    let r = read_resp(&mut tls_reader).await?;
    if !r.starts_with('2') {
        return Err(format!("message: {r}"));
    }

    let mid = r
        .split_whitespace()
        .last()
        .unwrap_or("unknown")
        .to_string();

    let _ = send_cmd(&mut tls_writer, "QUIT\r\n").await;
    Ok(mid)
}

async fn send_cmd<W: AsyncWriteExt + Unpin>(w: &mut W, cmd: &str) -> Result<(), String> {
    timeout(CMD_TIMEOUT, async {
        w.write_all(cmd.as_bytes())
            .await
            .map_err(|e| format!("write: {e}"))?;
        w.flush().await.map_err(|e| format!("flush: {e}"))
    })
    .await
    .map_err(|_| "send timeout".to_string())?
}

async fn read_resp<R: AsyncBufReadExt + Unpin>(r: &mut R) -> Result<String, String> {
    timeout(CMD_TIMEOUT, async {
        let mut full = String::new();
        loop {
            let mut line = String::new();
            let n = r
                .read_line(&mut line)
                .await
                .map_err(|e| format!("read: {e}"))?;
            if n == 0 {
                return Err("connection closed".to_string());
            }
            full.push_str(&line);
            if line.len() >= 4 && line.as_bytes()[3] == b' ' {
                break;
            }
        }
        Ok(full)
    })
    .await
    .map_err(|_| "read timeout".to_string())?
}

/// Rewrite the From header in MIME to match the authenticated SMTP user.
/// Adds Reply-To with the original sender so replies go back correctly.
fn rewrite_from(mime: &str, smtp_user: &str, original_from: &str) -> String {
    let mut result = String::with_capacity(mime.len() + 100);
    let mut added_reply_to = false;

    for line in mime.lines() {
        if line.starts_with("From:") || line.starts_with("from:") {
            // Replace From with the SMTP account
            result.push_str(&format!("From: Ferrum Mail <{smtp_user}>\r\n"));
            // Add Reply-To with original sender
            if !added_reply_to {
                result.push_str(&format!("Reply-To: {original_from}\r\n"));
                added_reply_to = true;
            }
        } else {
            result.push_str(line);
            result.push_str("\r\n");
        }
    }
    result
}
