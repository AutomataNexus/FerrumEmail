//! Native SMTP provider — sends email directly over SMTP with STARTTLS.
//!
//! No external email crates. Implements SMTP (RFC 5321), STARTTLS (RFC 3207),
//! SMTP AUTH PLAIN/LOGIN (RFC 4954), and MIME multipart message formatting.

use async_trait::async_trait;
use base64::Engine;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use crate::error::EmailError;
use crate::message::{EmailMessage, SendResult};
use crate::provider::EmailProvider;

/// SMTP authentication mechanism.
#[derive(Debug, Clone)]
pub enum SmtpAuth {
    /// AUTH PLAIN (username + password in one base64 blob).
    Plain { username: String, password: String },
    /// AUTH LOGIN (username and password sent separately).
    Login { username: String, password: String },
}

/// Native SMTP email provider.
///
/// Connects via TCP, upgrades to TLS with STARTTLS, authenticates,
/// and sends MIME-formatted email messages.
pub struct SmtpProvider {
    host: String,
    port: u16,
    auth: Option<SmtpAuth>,
    tls_config: Arc<rustls::ClientConfig>,
}

impl SmtpProvider {
    /// Create a new SMTP provider builder.
    pub fn builder() -> SmtpProviderBuilder {
        SmtpProviderBuilder {
            host: None,
            port: 587,
            username: None,
            password: None,
            auth_method: AuthMethod::Login,
            starttls: true,
        }
    }

    /// Build a MIME message from an EmailMessage.
    pub fn build_mime(message: &EmailMessage) -> String {
        let boundary = format!("ferrum-{:016x}", rand_u64());
        let mut mime = String::with_capacity(message.html.len() + 1024);

        // Headers
        mime.push_str(&format!("From: {}\r\n", message.from));
        for to in &message.to {
            mime.push_str(&format!("To: {}\r\n", to));
        }
        for cc in &message.cc {
            mime.push_str(&format!("Cc: {}\r\n", cc));
        }
        if let Some(ref reply_to) = message.reply_to {
            mime.push_str(&format!("Reply-To: {}\r\n", reply_to));
        }
        mime.push_str(&format!("Subject: {}\r\n", message.subject));
        mime.push_str("MIME-Version: 1.0\r\n");
        for (name, value) in &message.headers {
            mime.push_str(&format!("{}: {}\r\n", name, value));
        }

        let has_text = message.text.is_some();
        // Split inline images (with Content-ID) from regular attachments
        let inline_images: Vec<&crate::message::Attachment> = message
            .attachments
            .iter()
            .filter(|a| a.content_id.is_some())
            .collect();
        let regular_attachments: Vec<&crate::message::Attachment> = message
            .attachments
            .iter()
            .filter(|a| a.content_id.is_none())
            .collect();
        let has_inline = !inline_images.is_empty();
        let has_attachments = !regular_attachments.is_empty();

        // Helper: write the text+html alternative block
        let write_alternative = |mime: &mut String, alt_boundary: &str| {
            if let Some(ref text) = message.text {
                mime.push_str(&format!("--{alt_boundary}\r\n"));
                mime.push_str("Content-Type: text/plain; charset=UTF-8\r\n");
                mime.push_str("Content-Transfer-Encoding: 7bit\r\n\r\n");
                mime.push_str(text);
                mime.push_str("\r\n");
            }
            mime.push_str(&format!("--{alt_boundary}\r\n"));
            mime.push_str("Content-Type: text/html; charset=UTF-8\r\n");
            mime.push_str("Content-Transfer-Encoding: 7bit\r\n\r\n");
            mime.push_str(&message.html);
            mime.push_str("\r\n");
            mime.push_str(&format!("--{alt_boundary}--\r\n"));
        };

        // Helper: write a single attachment part
        let write_attachment = |mime: &mut String, att: &crate::message::Attachment| {
            mime.push_str(&format!(
                "Content-Type: {}; name=\"{}\"\r\n",
                att.content_type, att.filename
            ));
            mime.push_str("Content-Transfer-Encoding: base64\r\n");
            if let Some(ref cid) = att.content_id {
                mime.push_str(&format!("Content-ID: <{cid}>\r\n"));
                mime.push_str(&format!(
                    "Content-Disposition: inline; filename=\"{}\"\r\n\r\n",
                    att.filename
                ));
            } else {
                mime.push_str(&format!(
                    "Content-Disposition: attachment; filename=\"{}\"\r\n\r\n",
                    att.filename
                ));
            }
            let encoded = base64::engine::general_purpose::STANDARD.encode(&att.content);
            for chunk in encoded.as_bytes().chunks(76) {
                mime.push_str(std::str::from_utf8(chunk).unwrap_or(""));
                mime.push_str("\r\n");
            }
        };

        if has_attachments && has_inline {
            // multipart/mixed → (multipart/related → (alternative + inline)) + attachments
            let related_boundary = format!("ferrum-rel-{:016x}", rand_u64());
            let alt_boundary = format!("ferrum-alt-{:016x}", rand_u64());
            mime.push_str(&format!(
                "Content-Type: multipart/mixed; boundary=\"{boundary}\"\r\n\r\n"
            ));
            mime.push_str(&format!("--{boundary}\r\n"));
            mime.push_str(&format!(
                "Content-Type: multipart/related; boundary=\"{related_boundary}\"\r\n\r\n"
            ));
            mime.push_str(&format!("--{related_boundary}\r\n"));
            mime.push_str(&format!(
                "Content-Type: multipart/alternative; boundary=\"{alt_boundary}\"\r\n\r\n"
            ));
            write_alternative(&mut mime, &alt_boundary);
            for img in &inline_images {
                mime.push_str(&format!("--{related_boundary}\r\n"));
                write_attachment(&mut mime, img);
            }
            mime.push_str(&format!("--{related_boundary}--\r\n"));
            for att in &regular_attachments {
                mime.push_str(&format!("--{boundary}\r\n"));
                write_attachment(&mut mime, att);
            }
            mime.push_str(&format!("--{boundary}--\r\n"));
        } else if has_inline {
            // multipart/related → (alternative + inline)
            let alt_boundary = format!("ferrum-alt-{:016x}", rand_u64());
            mime.push_str(&format!(
                "Content-Type: multipart/related; boundary=\"{boundary}\"\r\n\r\n"
            ));
            mime.push_str(&format!("--{boundary}\r\n"));
            mime.push_str(&format!(
                "Content-Type: multipart/alternative; boundary=\"{alt_boundary}\"\r\n\r\n"
            ));
            write_alternative(&mut mime, &alt_boundary);
            for img in &inline_images {
                mime.push_str(&format!("--{boundary}\r\n"));
                write_attachment(&mut mime, img);
            }
            mime.push_str(&format!("--{boundary}--\r\n"));
        } else if has_attachments {
            // multipart/mixed wrapping multipart/alternative + attachments
            let alt_boundary = format!("ferrum-alt-{:016x}", rand_u64());
            mime.push_str(&format!(
                "Content-Type: multipart/mixed; boundary=\"{boundary}\"\r\n\r\n"
            ));
            mime.push_str(&format!("--{boundary}\r\n"));
            mime.push_str(&format!(
                "Content-Type: multipart/alternative; boundary=\"{alt_boundary}\"\r\n\r\n"
            ));
            write_alternative(&mut mime, &alt_boundary);
            for att in &regular_attachments {
                mime.push_str(&format!("--{boundary}\r\n"));
                write_attachment(&mut mime, att);
            }
            mime.push_str(&format!("--{boundary}--\r\n"));
        } else if has_text {
            // multipart/alternative: text + html
            mime.push_str(&format!(
                "Content-Type: multipart/alternative; boundary=\"{boundary}\"\r\n\r\n"
            ));
            mime.push_str(&format!("--{boundary}\r\n"));
            mime.push_str("Content-Type: text/plain; charset=UTF-8\r\n");
            mime.push_str("Content-Transfer-Encoding: 7bit\r\n\r\n");
            mime.push_str(message.text.as_deref().unwrap_or(""));
            mime.push_str("\r\n");
            mime.push_str(&format!("--{boundary}\r\n"));
            mime.push_str("Content-Type: text/html; charset=UTF-8\r\n");
            mime.push_str("Content-Transfer-Encoding: 7bit\r\n\r\n");
            mime.push_str(&message.html);
            mime.push_str("\r\n");
            mime.push_str(&format!("--{boundary}--\r\n"));
        } else {
            // HTML only
            mime.push_str("Content-Type: text/html; charset=UTF-8\r\n");
            mime.push_str("Content-Transfer-Encoding: 7bit\r\n\r\n");
            mime.push_str(&message.html);
            mime.push_str("\r\n");
        }

        mime
    }
}

#[async_trait]
impl EmailProvider for SmtpProvider {
    async fn send(&self, message: EmailMessage) -> Result<SendResult, EmailError> {
        // Connect TCP
        let addr = format!("{}:{}", self.host, self.port);
        let tcp = TcpStream::connect(&addr)
            .await
            .map_err(|e| EmailError::Provider(format!("SMTP connect to {addr} failed: {e}")))?;

        let (reader, mut writer) = tokio::io::split(tcp);
        let mut reader = BufReader::new(reader);

        // Read greeting
        let greeting = read_response(&mut reader).await?;
        if !greeting.starts_with('2') {
            return Err(EmailError::Provider(format!(
                "SMTP greeting rejected: {greeting}"
            )));
        }

        // EHLO
        send_command(&mut writer, "EHLO ferrum-email\r\n").await?;
        let ehlo_resp = read_response(&mut reader).await?;
        if !ehlo_resp.starts_with('2') {
            return Err(EmailError::Provider(format!("EHLO rejected: {ehlo_resp}")));
        }

        // STARTTLS
        send_command(&mut writer, "STARTTLS\r\n").await?;
        let starttls_resp = read_response(&mut reader).await?;
        if !starttls_resp.starts_with('2') {
            return Err(EmailError::Provider(format!(
                "STARTTLS rejected: {starttls_resp}"
            )));
        }

        // Upgrade to TLS
        let server_name = rustls::pki_types::ServerName::try_from(self.host.clone())
            .map_err(|e| EmailError::Provider(format!("invalid server name: {e}")))?;

        // Reunite the split halves for TLS upgrade
        let tcp = reader.into_inner().unsplit(writer);

        let tls_connector = tokio_rustls_connect(self.tls_config.clone());
        let tls_stream = tls_connector
            .connect(server_name, tcp)
            .await
            .map_err(|e| EmailError::Provider(format!("TLS handshake failed: {e}")))?;

        let (tls_reader, mut tls_writer) = tokio::io::split(tls_stream);
        let mut tls_reader = BufReader::new(tls_reader);

        // EHLO again after TLS
        send_command(&mut tls_writer, "EHLO ferrum-email\r\n").await?;
        let _ehlo2 = read_response(&mut tls_reader).await?;

        // AUTH
        if let Some(ref auth) = self.auth {
            match auth {
                SmtpAuth::Plain { username, password } => {
                    // AUTH PLAIN: \0username\0password base64-encoded
                    let plain = format!("\x00{username}\x00{password}");
                    let encoded =
                        base64::engine::general_purpose::STANDARD.encode(plain.as_bytes());
                    send_command(&mut tls_writer, &format!("AUTH PLAIN {encoded}\r\n")).await?;
                    let auth_resp = read_response(&mut tls_reader).await?;
                    if !auth_resp.starts_with('2') {
                        return Err(EmailError::Provider(format!(
                            "AUTH PLAIN failed: {auth_resp}"
                        )));
                    }
                }
                SmtpAuth::Login { username, password } => {
                    send_command(&mut tls_writer, "AUTH LOGIN\r\n").await?;
                    let prompt1 = read_response(&mut tls_reader).await?;
                    if !prompt1.starts_with('3') {
                        return Err(EmailError::Provider(format!(
                            "AUTH LOGIN rejected: {prompt1}"
                        )));
                    }
                    let user_b64 =
                        base64::engine::general_purpose::STANDARD.encode(username.as_bytes());
                    send_command(&mut tls_writer, &format!("{user_b64}\r\n")).await?;
                    let prompt2 = read_response(&mut tls_reader).await?;
                    if !prompt2.starts_with('3') {
                        return Err(EmailError::Provider(format!(
                            "AUTH LOGIN username rejected: {prompt2}"
                        )));
                    }
                    let pass_b64 =
                        base64::engine::general_purpose::STANDARD.encode(password.as_bytes());
                    send_command(&mut tls_writer, &format!("{pass_b64}\r\n")).await?;
                    let auth_resp = read_response(&mut tls_reader).await?;
                    if !auth_resp.starts_with('2') {
                        return Err(EmailError::Provider(format!(
                            "AUTH LOGIN failed: {auth_resp}"
                        )));
                    }
                }
            }
        }

        // MAIL FROM
        send_command(
            &mut tls_writer,
            &format!("MAIL FROM:<{}>\r\n", message.from.email),
        )
        .await?;
        let from_resp = read_response(&mut tls_reader).await?;
        if !from_resp.starts_with('2') {
            return Err(EmailError::Provider(format!(
                "MAIL FROM rejected: {from_resp}"
            )));
        }

        // RCPT TO (all recipients)
        let all_recipients: Vec<&str> = message
            .to
            .iter()
            .chain(message.cc.iter())
            .chain(message.bcc.iter())
            .map(|m| m.email.as_str())
            .collect();

        for rcpt in &all_recipients {
            send_command(&mut tls_writer, &format!("RCPT TO:<{rcpt}>\r\n")).await?;
            let rcpt_resp = read_response(&mut tls_reader).await?;
            if !rcpt_resp.starts_with('2') {
                return Err(EmailError::Provider(format!(
                    "RCPT TO <{rcpt}> rejected: {rcpt_resp}"
                )));
            }
        }

        // DATA
        send_command(&mut tls_writer, "DATA\r\n").await?;
        let data_resp = read_response(&mut tls_reader).await?;
        if !data_resp.starts_with('3') {
            return Err(EmailError::Provider(format!("DATA rejected: {data_resp}")));
        }

        // Send MIME message body
        let mime_body = Self::build_mime(&message);
        tls_writer
            .write_all(mime_body.as_bytes())
            .await
            .map_err(|e| EmailError::Provider(format!("failed to write message body: {e}")))?;

        // End with \r\n.\r\n
        send_command(&mut tls_writer, "\r\n.\r\n").await?;
        let msg_resp = read_response(&mut tls_reader).await?;
        if !msg_resp.starts_with('2') {
            return Err(EmailError::Provider(format!(
                "message rejected: {msg_resp}"
            )));
        }

        // Extract message ID from response if available
        let message_id =
            extract_message_id(&msg_resp).unwrap_or_else(|| format!("smtp-{:016x}", rand_u64()));

        // QUIT
        let _ = send_command(&mut tls_writer, "QUIT\r\n").await;
        let _ = read_response(&mut tls_reader).await;

        Ok(SendResult {
            message_id,
            provider: "smtp".to_string(),
        })
    }
}

// ---------------------------------------------------------------------------
// SMTP provider builder
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum AuthMethod {
    Plain,
    Login,
}

/// Builder for `SmtpProvider`.
pub struct SmtpProviderBuilder {
    host: Option<String>,
    port: u16,
    username: Option<String>,
    password: Option<String>,
    auth_method: AuthMethod,
    starttls: bool,
}

impl SmtpProviderBuilder {
    /// Set the SMTP server hostname.
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Set the SMTP server port (default: 587).
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set authentication credentials.
    pub fn credentials(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    /// Use AUTH PLAIN (default is AUTH LOGIN).
    pub fn auth_plain(mut self) -> Self {
        self.auth_method = AuthMethod::Plain;
        self
    }

    /// Use AUTH LOGIN (default).
    pub fn auth_login(mut self) -> Self {
        self.auth_method = AuthMethod::Login;
        self
    }

    /// Enable or disable STARTTLS (default: true).
    pub fn starttls(mut self, enabled: bool) -> Self {
        self.starttls = enabled;
        self
    }

    /// Build the SMTP provider.
    pub fn build(self) -> Result<SmtpProvider, EmailError> {
        let host = self
            .host
            .ok_or_else(|| EmailError::MissingField("SMTP host is required".into()))?;

        let auth = match (self.username, self.password) {
            (Some(username), Some(password)) => Some(match self.auth_method {
                AuthMethod::Plain => SmtpAuth::Plain { username, password },
                AuthMethod::Login => SmtpAuth::Login { username, password },
            }),
            _ => None,
        };

        // Build TLS config with system root certificates
        let root_store =
            rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let tls_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(SmtpProvider {
            host,
            port: self.port,
            auth,
            tls_config: Arc::new(tls_config),
        })
    }
}

// ---------------------------------------------------------------------------
// TLS connector (tokio + rustls without tokio-rustls dep)
// ---------------------------------------------------------------------------

/// Minimal TLS connector using rustls directly over a tokio TcpStream.
struct TlsConnector {
    config: Arc<rustls::ClientConfig>,
}

fn tokio_rustls_connect(config: Arc<rustls::ClientConfig>) -> TlsConnector {
    TlsConnector { config }
}

impl TlsConnector {
    async fn connect(
        self,
        server_name: rustls::pki_types::ServerName<'static>,
        tcp: TcpStream,
    ) -> Result<tokio_rustls::client::TlsStream<TcpStream>, std::io::Error> {
        let connector = tokio_rustls::TlsConnector::from(self.config);
        connector.connect(server_name, tcp).await
    }
}

// ---------------------------------------------------------------------------
// SMTP protocol helpers
// ---------------------------------------------------------------------------

async fn send_command<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    command: &str,
) -> Result<(), EmailError> {
    writer
        .write_all(command.as_bytes())
        .await
        .map_err(|e| EmailError::Provider(format!("SMTP write failed: {e}")))?;
    writer
        .flush()
        .await
        .map_err(|e| EmailError::Provider(format!("SMTP flush failed: {e}")))?;
    Ok(())
}

async fn read_response<R: AsyncBufReadExt + Unpin>(reader: &mut R) -> Result<String, EmailError> {
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
        // Multi-line responses have '-' after the status code; last line has ' '
        if line.len() >= 4 && line.as_bytes()[3] == b' ' {
            break;
        }
    }
    Ok(full_response)
}

fn extract_message_id(response: &str) -> Option<String> {
    // Many SMTP servers return the message ID in the 250 OK response
    // e.g., "250 2.0.0 Ok: queued as ABC123"
    response
        .split_whitespace()
        .last()
        .map(|s| s.trim().to_string())
}

/// Simple pseudo-random u64 for boundary generation (not crypto, just uniqueness).
fn rand_u64() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    t.as_nanos() as u64 ^ (t.as_secs().wrapping_mul(6364136223846793005))
}
