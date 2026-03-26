//! Email message types — Mailbox, EmailMessage, Attachment, SendResult.

use std::fmt;
use std::str::FromStr;

use crate::error::EmailError;

/// An email address with an optional display name.
///
/// Formats as `"Name <email@domain.com>"` or just `"email@domain.com"`.
#[derive(Debug, Clone, PartialEq)]
pub struct Mailbox {
    pub name: Option<String>,
    pub email: String,
}

impl Mailbox {
    /// Create a new Mailbox with name and email.
    pub fn new(name: impl Into<String>, email: impl Into<String>) -> Self {
        Mailbox {
            name: Some(name.into()),
            email: email.into(),
        }
    }

    /// Create a new Mailbox with just an email address.
    pub fn address(email: impl Into<String>) -> Self {
        Mailbox {
            name: None,
            email: email.into(),
        }
    }
}

impl fmt::Display for Mailbox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{} <{}>", name, self.email),
            None => write!(f, "{}", self.email),
        }
    }
}

impl FromStr for Mailbox {
    type Err = EmailError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        // Try to parse "Name <email@domain.com>" format
        if let Some(angle_start) = s.rfind('<')
            && let Some(angle_end) = s.rfind('>')
            && angle_end > angle_start
        {
            let email = s[angle_start + 1..angle_end].trim().to_string();
            let name = s[..angle_start].trim();
            let name = name.trim_matches('"').trim();

            if email.is_empty() || !email.contains('@') {
                return Err(EmailError::InvalidAddress(s.to_string()));
            }

            return Ok(Mailbox {
                name: if name.is_empty() {
                    None
                } else {
                    Some(name.to_string())
                },
                email,
            });
        }

        // Plain email address
        if s.contains('@') && !s.contains(' ') {
            Ok(Mailbox {
                name: None,
                email: s.to_string(),
            })
        } else {
            Err(EmailError::InvalidAddress(s.to_string()))
        }
    }
}

impl From<&str> for Mailbox {
    fn from(s: &str) -> Self {
        s.parse().unwrap_or_else(|_| Mailbox {
            name: None,
            email: s.to_string(),
        })
    }
}

impl From<String> for Mailbox {
    fn from(s: String) -> Self {
        Mailbox::from(s.as_str())
    }
}

/// A tag for categorizing emails (provider-specific).
#[derive(Debug, Clone, PartialEq)]
pub struct EmailTag {
    pub name: String,
    pub value: String,
}

impl EmailTag {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        EmailTag {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// An email attachment.
#[derive(Debug, Clone)]
pub struct Attachment {
    pub filename: String,
    pub content: Vec<u8>,
    pub content_type: String,
}

impl Attachment {
    /// Create an attachment from raw bytes.
    pub fn from_bytes(
        filename: impl Into<String>,
        content: Vec<u8>,
        content_type: impl Into<String>,
    ) -> Self {
        Attachment {
            filename: filename.into(),
            content,
            content_type: content_type.into(),
        }
    }
}

/// A complete email message ready to be sent via a provider.
#[derive(Debug, Clone)]
pub struct EmailMessage {
    pub from: Mailbox,
    pub to: Vec<Mailbox>,
    pub cc: Vec<Mailbox>,
    pub bcc: Vec<Mailbox>,
    pub reply_to: Option<Mailbox>,
    pub subject: String,
    pub html: String,
    pub text: Option<String>,
    pub attachments: Vec<Attachment>,
    pub headers: Vec<(String, String)>,
    pub tags: Vec<EmailTag>,
}

impl Default for EmailMessage {
    fn default() -> Self {
        EmailMessage {
            from: Mailbox::address("noreply@example.com"),
            to: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            reply_to: None,
            subject: String::new(),
            html: String::new(),
            text: None,
            attachments: Vec::new(),
            headers: Vec::new(),
            tags: Vec::new(),
        }
    }
}

/// The result of a successful email send.
#[derive(Debug, Clone)]
pub struct SendResult {
    /// The provider-assigned message ID.
    pub message_id: String,
    /// The name of the provider that sent the message.
    pub provider: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mailbox_parse_name_and_email() {
        let mb: Mailbox = "Andrew <andrew@example.com>".parse().unwrap();
        assert_eq!(mb.name, Some("Andrew".to_string()));
        assert_eq!(mb.email, "andrew@example.com");
    }

    #[test]
    fn test_mailbox_parse_email_only() {
        let mb: Mailbox = "andrew@example.com".parse().unwrap();
        assert_eq!(mb.name, None);
        assert_eq!(mb.email, "andrew@example.com");
    }

    #[test]
    fn test_mailbox_display_with_name() {
        let mb = Mailbox::new("Andrew", "andrew@example.com");
        assert_eq!(mb.to_string(), "Andrew <andrew@example.com>");
    }

    #[test]
    fn test_mailbox_display_email_only() {
        let mb = Mailbox::address("andrew@example.com");
        assert_eq!(mb.to_string(), "andrew@example.com");
    }

    #[test]
    fn test_mailbox_from_str() {
        let mb = Mailbox::from("AutomataNexus <no-reply@automatanexus.com>");
        assert_eq!(mb.name, Some("AutomataNexus".to_string()));
        assert_eq!(mb.email, "no-reply@automatanexus.com");
    }
}
