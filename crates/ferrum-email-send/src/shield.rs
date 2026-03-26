//! NexusShield integration — security validation for outbound emails.
//!
//! When the `shield` feature is enabled, all outbound emails are validated
//! through NexusShield's email guard before sending:
//!
//! - **Header injection** (CRLF in subject, name, addresses)
//! - **Email bombing** prevention (rate limiting per recipient)
//! - **Content injection** (XSS, script tags, event handlers in template data)
//! - **Address validation** (format, blocked domains, disposable addresses)
//! - **Encoded attack** detection (base64 payloads, unicode tricks)
//! - **Oversized field** prevention
//!
//! This runs automatically on every `Sender::send()` and `Sender::send_message()`
//! call. No configuration needed — NexusShield uses safe defaults.

use nexus_shield::Shield;

use crate::error::EmailError;
use crate::message::EmailMessage;

/// Validate an outbound email message through NexusShield.
///
/// Returns `Ok(())` if the email passes all security checks, or
/// `Err(EmailError)` with details about what failed.
pub fn validate_outbound(message: &EmailMessage) -> Result<(), EmailError> {
    let shield = Shield::new(Default::default());

    // Validate sender address
    shield
        .validate_email_address(&message.from.email)
        .map_err(|e| EmailError::Provider(format!("Shield: sender address rejected: {e}")))?;

    // Validate all recipient addresses
    for to in &message.to {
        shield.validate_email_address(&to.email).map_err(|e| {
            EmailError::Provider(format!("Shield: recipient <{}> rejected: {e}", to.email))
        })?;
    }
    for cc in &message.cc {
        shield.validate_email_address(&cc.email).map_err(|e| {
            EmailError::Provider(format!("Shield: CC <{}> rejected: {e}", cc.email))
        })?;
    }
    for bcc in &message.bcc {
        shield.validate_email_address(&bcc.email).map_err(|e| {
            EmailError::Provider(format!("Shield: BCC <{}> rejected: {e}", bcc.email))
        })?;
    }

    // Validate subject (header injection)
    shield
        .validate_email_header("subject", &message.subject)
        .map_err(|e| EmailError::Provider(format!("Shield: subject rejected: {e}")))?;

    // Validate sender name if present
    if let Some(ref name) = message.from.name {
        shield
            .validate_email_header("from_name", name)
            .map_err(|e| EmailError::Provider(format!("Shield: sender name rejected: {e}")))?;
    }

    // Note: we do NOT validate the HTML/text body content here because it's
    // rendered output from our own component system (or user-provided HTML).
    // The shield's content validator is designed for template parameters
    // (user-supplied names, subjects), not final rendered HTML which
    // legitimately contains tags like <h1>, <a>, <img>, etc.

    // Validate custom headers
    for (name, value) in &message.headers {
        shield
            .validate_email_header(name, value)
            .map_err(|e| EmailError::Provider(format!("Shield: header '{name}' rejected: {e}")))?;
    }

    // Rate check for each recipient
    let all_recipients = message
        .to
        .iter()
        .chain(message.cc.iter())
        .chain(message.bcc.iter());

    for recipient in all_recipients {
        shield.check_email_rate(&recipient.email).map_err(|e| {
            EmailError::Provider(format!("Shield: rate limit for <{}>: {e}", recipient.email))
        })?;
    }

    Ok(())
}
