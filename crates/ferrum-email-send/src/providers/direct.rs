//! NexusRelay — direct MX delivery provider.
//!
//! Resolves recipient MX records via DNS, connects directly to the recipient's
//! mail server, upgrades to TLS (STARTTLS), and delivers. No third-party relay
//! needed — Ferrum is its own outbound SMTP relay.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

use crate::error::EmailError;
use crate::message::{EmailMessage, SendResult};
use crate::provider::EmailProvider;

// Re-use SMTP helpers from the smtp module
use super::smtp::SmtpProvider;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const CMD_TIMEOUT: Duration = Duration::from_secs(30);
const DATA_TIMEOUT: Duration = Duration::from_secs(60);

/// Direct MX delivery provider — Ferrum's own outbound relay ("NexusRelay").
///
/// For each recipient domain, resolves MX records via DNS, connects to the
/// highest-priority mail server, upgrades to TLS if available, and delivers.
pub struct DirectMxProvider {
    /// EHLO hostname (should be the sending server's FQDN)
    ehlo_domain: String,
    tls_config: Arc<rustls::ClientConfig>,
}

impl DirectMxProvider {
    /// Create a new direct MX provider.
    ///
    /// `ehlo_domain` should be the FQDN of your mail server (e.g., "mail.ferrum-mail.com").
    pub fn new(ehlo_domain: impl Into<String>) -> Self {
        let root_store =
            rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let tls_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Self {
            ehlo_domain: ehlo_domain.into(),
            tls_config: Arc::new(tls_config),
        }
    }

    /// Resolve MX records for a domain, sorted by priority (lowest first).
    async fn resolve_mx(domain: &str) -> Result<Vec<String>, EmailError> {
        // Use tokio's DNS resolver to get MX records
        // We shell out to `dig` since tokio doesn't have native MX support
        // Sanitize domain to prevent command injection
        if domain.is_empty()
            || domain.len() > 253
            || !domain
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
        {
            return Err(EmailError::Provider(format!(
                "Invalid domain for MX lookup: {domain}"
            )));
        }

        let output = tokio::process::Command::new("dig")
            .args(["+short", "MX", domain])
            .output()
            .await
            .map_err(|e| EmailError::Provider(format!("DNS MX lookup failed for {domain}: {e}")))?;

        if !output.status.success() {
            return Err(EmailError::Provider(format!(
                "DNS MX lookup failed for {domain}"
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut mx_records: Vec<(u16, String)> = stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() == 2 {
                    let priority = parts[0].parse::<u16>().ok()?;
                    let host = parts[1].trim_end_matches('.').to_string();
                    Some((priority, host))
                } else {
                    None
                }
            })
            .collect();

        mx_records.sort_by_key(|(p, _)| *p);

        if mx_records.is_empty() {
            // Fall back to A record (the domain itself)
            Ok(vec![domain.to_string()])
        } else {
            Ok(mx_records.into_iter().map(|(_, h)| h).collect())
        }
    }

    /// Deliver a message to a single recipient's MX server.
    async fn deliver_to_mx(
        &self,
        mx_host: &str,
        from: &str,
        to: &str,
        mime_body: &str,
    ) -> Result<String, EmailError> {
        // Connect with timeout
        let addr = format!("{mx_host}:25");
        let tcp = timeout(CONNECT_TIMEOUT, TcpStream::connect(&addr))
            .await
            .map_err(|_| {
                EmailError::Provider(format!("connection to {addr} timed out after 15s"))
            })?
            .map_err(|e| EmailError::Provider(format!("connect to {addr} failed: {e}")))?;

        let (reader, mut writer) = tokio::io::split(tcp);
        let mut reader = BufReader::new(reader);

        // Greeting
        let greeting = timed_read(&mut reader, CMD_TIMEOUT).await?;
        if !greeting.starts_with('2') {
            return Err(EmailError::Provider(format!(
                "MX {mx_host} greeting rejected: {greeting}"
            )));
        }

        // EHLO
        timed_send(&mut writer, &format!("EHLO {}\r\n", self.ehlo_domain), CMD_TIMEOUT).await?;
        let ehlo_resp = timed_read(&mut reader, CMD_TIMEOUT).await?;
        if !ehlo_resp.starts_with('2') {
            return Err(EmailError::Provider(format!(
                "MX {mx_host} EHLO rejected: {ehlo_resp}"
            )));
        }

        // Try STARTTLS if advertised
        let supports_starttls = ehlo_resp.to_uppercase().contains("STARTTLS");

        if supports_starttls {
            timed_send(&mut writer, "STARTTLS\r\n", CMD_TIMEOUT).await?;
            let starttls_resp = timed_read(&mut reader, CMD_TIMEOUT).await?;

            if starttls_resp.starts_with('2') {
                // Upgrade to TLS
                let server_name =
                    rustls::pki_types::ServerName::try_from(mx_host.to_string())
                        .map_err(|e| EmailError::Provider(format!("invalid server name: {e}")))?;

                let tcp = reader.into_inner().unsplit(writer);
                let connector = tokio_rustls::TlsConnector::from(self.tls_config.clone());
                let tls_stream = timeout(CONNECT_TIMEOUT, connector.connect(server_name, tcp))
                    .await
                    .map_err(|_| EmailError::Provider("TLS handshake timed out".into()))?
                    .map_err(|e| {
                        EmailError::Provider(format!("TLS handshake with {mx_host} failed: {e}"))
                    })?;

                let (tls_reader, mut tls_writer) = tokio::io::split(tls_stream);
                let mut tls_reader = BufReader::new(tls_reader);

                // EHLO again after TLS
                timed_send(
                    &mut tls_writer,
                    &format!("EHLO {}\r\n", self.ehlo_domain),
                    CMD_TIMEOUT,
                )
                .await?;
                let _ = timed_read(&mut tls_reader, CMD_TIMEOUT).await?;

                // Send envelope + data over TLS
                return self
                    .send_envelope(&mut tls_reader, &mut tls_writer, from, to, mime_body, mx_host)
                    .await;
            }
            // If STARTTLS failed, fall through to plaintext
        }

        // Send envelope + data (plaintext — some servers don't support TLS)
        self.send_envelope(&mut reader, &mut writer, from, to, mime_body, mx_host)
            .await
    }

    /// Send the SMTP envelope (MAIL FROM, RCPT TO, DATA) over an established connection.
    async fn send_envelope<R, W>(
        &self,
        reader: &mut R,
        writer: &mut W,
        from: &str,
        to: &str,
        mime_body: &str,
        mx_host: &str,
    ) -> Result<String, EmailError>
    where
        R: AsyncBufReadExt + Unpin,
        W: AsyncWriteExt + Unpin,
    {
        // MAIL FROM
        timed_send(writer, &format!("MAIL FROM:<{from}>\r\n"), CMD_TIMEOUT).await?;
        let from_resp = timed_read(reader, CMD_TIMEOUT).await?;
        if !from_resp.starts_with('2') {
            return Err(EmailError::Provider(format!(
                "MX {mx_host} MAIL FROM rejected: {from_resp}"
            )));
        }

        // RCPT TO
        timed_send(writer, &format!("RCPT TO:<{to}>\r\n"), CMD_TIMEOUT).await?;
        let rcpt_resp = timed_read(reader, CMD_TIMEOUT).await?;
        if !rcpt_resp.starts_with('2') {
            return Err(EmailError::Provider(format!(
                "MX {mx_host} RCPT TO <{to}> rejected: {rcpt_resp}"
            )));
        }

        // DATA
        timed_send(writer, "DATA\r\n", CMD_TIMEOUT).await?;
        let data_resp = timed_read(reader, CMD_TIMEOUT).await?;
        if !data_resp.starts_with('3') {
            return Err(EmailError::Provider(format!(
                "MX {mx_host} DATA rejected: {data_resp}"
            )));
        }

        // Send MIME body
        timeout(DATA_TIMEOUT, writer.write_all(mime_body.as_bytes()))
            .await
            .map_err(|_| EmailError::Provider("timed out writing message body".into()))?
            .map_err(|e| EmailError::Provider(format!("write body failed: {e}")))?;

        // End message
        timed_send(writer, "\r\n.\r\n", DATA_TIMEOUT).await?;
        let msg_resp = timed_read(reader, CMD_TIMEOUT).await?;
        if !msg_resp.starts_with('2') {
            return Err(EmailError::Provider(format!(
                "MX {mx_host} message rejected: {msg_resp}"
            )));
        }

        let message_id = msg_resp
            .split_whitespace()
            .last()
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| format!("nexus-{:016x}", rand_u64()));

        // QUIT (best effort)
        let _ = timed_send(writer, "QUIT\r\n", Duration::from_secs(5)).await;

        Ok(message_id)
    }
}

#[async_trait]
impl EmailProvider for DirectMxProvider {
    async fn send(&self, message: EmailMessage) -> Result<SendResult, EmailError> {
        let from_email = &message.from.email;
        let mime_body = SmtpProvider::build_mime(&message);

        // Group recipients by domain
        let all_recipients: Vec<&str> = message
            .to
            .iter()
            .chain(message.cc.iter())
            .chain(message.bcc.iter())
            .map(|m| m.email.as_str())
            .collect();

        let mut last_message_id = String::new();

        for rcpt in &all_recipients {
            let domain = rcpt
                .rsplit('@')
                .next()
                .ok_or_else(|| EmailError::Provider(format!("invalid email: {rcpt}")))?;

            let mx_hosts = Self::resolve_mx(domain).await?;

            let mut delivered = false;
            let mut last_err = String::new();

            for mx in &mx_hosts {
                match self.deliver_to_mx(mx, from_email, rcpt, &mime_body).await {
                    Ok(mid) => {
                        last_message_id = mid;
                        delivered = true;
                        break;
                    }
                    Err(e) => {
                        last_err = e.to_string();
                        // Try next MX
                        continue;
                    }
                }
            }

            if !delivered {
                return Err(EmailError::Provider(format!(
                    "delivery to <{rcpt}> failed (tried {} MX hosts): {last_err}",
                    mx_hosts.len()
                )));
            }
        }

        Ok(SendResult {
            message_id: last_message_id,
            provider: "nexus-relay".to_string(),
        })
    }
}

// ── Timeout helpers ──

async fn timed_send<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    cmd: &str,
    dur: Duration,
) -> Result<(), EmailError> {
    timeout(dur, async {
        writer
            .write_all(cmd.as_bytes())
            .await
            .map_err(|e| EmailError::Provider(format!("SMTP write failed: {e}")))?;
        writer
            .flush()
            .await
            .map_err(|e| EmailError::Provider(format!("SMTP flush failed: {e}")))?;
        Ok(())
    })
    .await
    .map_err(|_| EmailError::Provider("SMTP send timed out".into()))?
}

async fn timed_read<R: AsyncBufReadExt + Unpin>(
    reader: &mut R,
    dur: Duration,
) -> Result<String, EmailError> {
    timeout(dur, async {
        let mut full_response = String::new();
        loop {
            let mut line = String::new();
            let n = reader
                .read_line(&mut line)
                .await
                .map_err(|e| EmailError::Provider(format!("SMTP read failed: {e}")))?;
            if n == 0 {
                return Err(EmailError::Provider(
                    "SMTP connection closed unexpectedly".into(),
                ));
            }
            full_response.push_str(&line);
            if line.len() >= 4 && line.as_bytes()[3] == b' ' {
                break;
            }
        }
        Ok(full_response)
    })
    .await
    .map_err(|_| EmailError::Provider("SMTP read timed out".into()))?
}

fn rand_u64() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    t.as_nanos() as u64 ^ (t.as_secs().wrapping_mul(6364136223846793005))
}
