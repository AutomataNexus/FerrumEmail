//! NexusRelay — direct MX SMTP delivery with DKIM signing.
//!
//! Runs on a machine with outbound port 25 access (Vultr).
//! Accepts HTTP relay requests over Tailscale, resolves recipient MX records,
//! signs with DKIM, and delivers directly to each recipient's mail server.

use axum::{Router, extract::Json, http::StatusCode, routing::post};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const CMD_TIMEOUT: Duration = Duration::from_secs(30);
const DATA_TIMEOUT: Duration = Duration::from_secs(60);

struct RelayConfig {
    ehlo_domain: String,
    dkim_key_path: String,
    dkim_domain: String,
    dkim_selector: String,
    tls_config: Arc<rustls::ClientConfig>,
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
    delivered: usize,
    failed: Vec<String>,
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

    let root_store =
        rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let tls_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let config = Arc::new(RelayConfig {
        ehlo_domain: std::env::var("EHLO_DOMAIN")
            .unwrap_or_else(|_| "relay.ferrum-mail.com".into()),
        dkim_key_path: std::env::var("DKIM_KEY_PATH")
            .unwrap_or_else(|_| "/var/lib/ferrum-email/dkim/ferrum-mail.com.private".into()),
        dkim_domain: std::env::var("DKIM_DOMAIN").unwrap_or_else(|_| "ferrum-mail.com".into()),
        dkim_selector: std::env::var("DKIM_SELECTOR").unwrap_or_else(|_| "ferrum".into()),
        tls_config: Arc::new(tls_config),
    });

    tracing::info!(
        "NexusRelay v0.2 — direct MX delivery, DKIM domain={} selector={}",
        config.dkim_domain,
        config.dkim_selector
    );

    // Validate DKIM key exists
    if !std::path::Path::new(&config.dkim_key_path).exists() {
        tracing::warn!(
            "DKIM private key not found at {} — emails will NOT be signed",
            config.dkim_key_path
        );
    } else {
        tracing::info!("DKIM key loaded: {}", config.dkim_key_path);
    }

    let app = Router::new()
        .route("/relay", post(handle_relay))
        .route("/health", axum::routing::get(|| async { "ok" }))
        .layer(axum::extract::DefaultBodyLimit::max(26_214_400))
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

    // Sign with DKIM
    let signed_body = match sign_dkim(&config, &req.mime_body) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("DKIM signing failed: {e} — sending unsigned");
            req.mime_body.clone()
        }
    };

    let mut delivered = 0;
    let mut failed = Vec::new();
    let mut last_mid = String::new();

    // Group recipients by domain for efficient delivery
    let mut by_domain: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for rcpt in &req.to {
        if let Some(domain) = rcpt.rsplit('@').next() {
            by_domain
                .entry(domain.to_lowercase())
                .or_default()
                .push(rcpt.clone());
        }
    }

    for (domain, recipients) in by_domain {
        match deliver_to_domain(&config, &req.from, &domain, &recipients, &signed_body).await {
            Ok(mid) => {
                delivered += recipients.len();
                last_mid = mid;
            }
            Err(e) => {
                tracing::error!("Delivery to {domain} failed: {e}");
                for r in recipients {
                    failed.push(format!("{r}: {e}"));
                }
            }
        }
    }

    if delivered == 0 && !failed.is_empty() {
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(RelayError {
                error: failed.join("; "),
            }),
        ));
    }

    Ok(Json(RelayResponse {
        message_id: last_mid,
        status: if failed.is_empty() {
            "sent".into()
        } else {
            "partial".into()
        },
        delivered,
        failed,
    }))
}

/// DKIM sign the MIME body using mail-auth.
fn sign_dkim(config: &RelayConfig, mime_body: &str) -> Result<String, String> {
    use mail_auth::common::crypto::{RsaKey, Sha256};
    use mail_auth::common::headers::HeaderWriter;
    use mail_auth::dkim::DkimSigner;

    let pem = std::fs::read_to_string(&config.dkim_key_path)
        .map_err(|e| format!("read DKIM key: {e}"))?;

    let pk = RsaKey::<Sha256>::from_rsa_pem(&pem)
        .or_else(|_| RsaKey::<Sha256>::from_pkcs8_pem(&pem))
        .map_err(|e| format!("parse DKIM key: {e:?}"))?;

    let signer = DkimSigner::from_key(pk)
        .domain(&config.dkim_domain)
        .selector(&config.dkim_selector)
        .headers(["From", "To", "Subject", "Date", "Message-ID"]);

    let signature = signer
        .sign(mime_body.as_bytes())
        .map_err(|e| format!("DKIM sign: {e:?}"))?;

    // Prepend the signature header to the message
    let mut header = Vec::with_capacity(512);
    signature.write_header(&mut header);
    let header_str = String::from_utf8(header).map_err(|e| format!("DKIM header utf8: {e}"))?;
    Ok(format!("{header_str}{mime_body}"))
}

/// Resolve hostname to IPv4 address using explicit resolver.
async fn resolve_ipv4(host: &str) -> Result<String, String> {
    let output = tokio::process::Command::new("dig")
        .args(["@8.8.8.8", "+short", "+time=5", "+tries=2", "A", host])
        .output()
        .await
        .map_err(|e| format!("dig A {host}: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .find(|l| l.chars().all(|c| c.is_ascii_digit() || c == '.'))
        .map(|s| s.to_string())
        .ok_or_else(|| format!("no A record for {host}"))
}

/// Resolve MX records for a domain (using explicit resolver).
async fn resolve_mx(domain: &str) -> Result<Vec<String>, String> {
    if domain.is_empty()
        || domain.len() > 253
        || !domain
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    {
        return Err(format!("invalid domain: {domain}"));
    }

    let output = tokio::process::Command::new("dig")
        .args(["@8.8.8.8", "+short", "+time=5", "+tries=2", "MX", domain])
        .output()
        .await
        .map_err(|e| format!("dig: {e}"))?;

    if !output.status.success() {
        return Err(format!("dig failed for {domain}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut mx: Vec<(u16, String)> = stdout
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 2 {
                let prio = parts[0].parse::<u16>().ok()?;
                let host = parts[1].trim_end_matches('.').to_string();
                Some((prio, host))
            } else {
                None
            }
        })
        .collect();
    mx.sort_by_key(|(p, _)| *p);

    if mx.is_empty() {
        // Fall back to A record (domain itself acts as MX)
        Ok(vec![domain.to_string()])
    } else {
        Ok(mx.into_iter().map(|(_, h)| h).collect())
    }
}

/// Deliver a message to all recipients in one domain (one connection).
async fn deliver_to_domain(
    config: &RelayConfig,
    from: &str,
    domain: &str,
    recipients: &[String],
    mime_body: &str,
) -> Result<String, String> {
    let mx_hosts = resolve_mx(domain).await?;
    let mut last_err = String::new();

    for mx_host in &mx_hosts {
        match try_deliver(config, mx_host, from, recipients, mime_body).await {
            Ok(mid) => return Ok(mid),
            Err(e) => {
                tracing::warn!("MX {mx_host} failed: {e}");
                last_err = e;
            }
        }
    }
    Err(format!("all MX hosts failed: {last_err}"))
}

async fn try_deliver(
    config: &RelayConfig,
    mx_host: &str,
    from: &str,
    recipients: &[String],
    mime_body: &str,
) -> Result<String, String> {
    // Pre-resolve hostname to IP (system resolver may be unreliable)
    let ip = resolve_ipv4(mx_host).await?;
    let addr = format!("{ip}:25");
    let tcp = timeout(CONNECT_TIMEOUT, TcpStream::connect(&addr))
        .await
        .map_err(|_| format!("timeout connecting to {mx_host} ({addr})"))?
        .map_err(|e| format!("connect {mx_host} ({addr}): {e}"))?;

    let (reader, mut writer) = tokio::io::split(tcp);
    let mut reader = BufReader::new(reader);

    // Greeting
    let greeting = read_resp(&mut reader).await?;
    if !greeting.starts_with('2') {
        return Err(format!("greeting: {greeting}"));
    }

    // EHLO
    send_cmd(&mut writer, &format!("EHLO {}\r\n", config.ehlo_domain)).await?;
    let ehlo = read_resp(&mut reader).await?;
    if !ehlo.starts_with('2') {
        return Err(format!("EHLO: {ehlo}"));
    }

    let supports_tls = ehlo.to_uppercase().contains("STARTTLS");

    if supports_tls {
        send_cmd(&mut writer, "STARTTLS\r\n").await?;
        let tls_resp = read_resp(&mut reader).await?;
        if tls_resp.starts_with('2') {
            let server_name = rustls::pki_types::ServerName::try_from(mx_host.to_string())
                .map_err(|e| format!("bad server name: {e}"))?;
            let tcp = reader.into_inner().unsplit(writer);
            let connector = tokio_rustls::TlsConnector::from(config.tls_config.clone());
            let tls_stream = timeout(CONNECT_TIMEOUT, connector.connect(server_name, tcp))
                .await
                .map_err(|_| "TLS handshake timeout".to_string())?
                .map_err(|e| format!("TLS handshake: {e}"))?;

            let (tls_reader, mut tls_writer) = tokio::io::split(tls_stream);
            let mut tls_reader = BufReader::new(tls_reader);

            // EHLO again after TLS
            send_cmd(&mut tls_writer, &format!("EHLO {}\r\n", config.ehlo_domain)).await?;
            let _ = read_resp(&mut tls_reader).await?;

            return send_envelope(&mut tls_reader, &mut tls_writer, from, recipients, mime_body)
                .await;
        }
    }

    send_envelope(&mut reader, &mut writer, from, recipients, mime_body).await
}

async fn send_envelope<R, W>(
    reader: &mut R,
    writer: &mut W,
    from: &str,
    recipients: &[String],
    mime_body: &str,
) -> Result<String, String>
where
    R: AsyncBufReadExt + Unpin,
    W: AsyncWriteExt + Unpin,
{
    // MAIL FROM — preserve original sender
    send_cmd(writer, &format!("MAIL FROM:<{from}>\r\n")).await?;
    let r = read_resp(reader).await?;
    if !r.starts_with('2') {
        return Err(format!("MAIL FROM: {r}"));
    }

    for rcpt in recipients {
        send_cmd(writer, &format!("RCPT TO:<{rcpt}>\r\n")).await?;
        let r = read_resp(reader).await?;
        if !r.starts_with('2') {
            return Err(format!("RCPT TO <{rcpt}>: {r}"));
        }
    }

    send_cmd(writer, "DATA\r\n").await?;
    let r = read_resp(reader).await?;
    if !r.starts_with('3') {
        return Err(format!("DATA: {r}"));
    }

    timeout(DATA_TIMEOUT, writer.write_all(mime_body.as_bytes()))
        .await
        .map_err(|_| "write body timeout".to_string())?
        .map_err(|e| format!("write body: {e}"))?;

    send_cmd(writer, "\r\n.\r\n").await?;
    let r = read_resp(reader).await?;
    if !r.starts_with('2') {
        return Err(format!("message: {r}"));
    }

    let mid = r
        .split_whitespace()
        .last()
        .unwrap_or("unknown")
        .to_string();

    let _ = send_cmd(writer, "QUIT\r\n").await;
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
