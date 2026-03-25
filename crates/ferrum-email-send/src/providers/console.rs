//! Console provider — prints emails to stdout for development and testing.

use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::EmailError;
use crate::message::{EmailMessage, SendResult};
use crate::provider::EmailProvider;

static CONSOLE_MSG_COUNTER: AtomicU64 = AtomicU64::new(1);

/// A provider that prints emails to stdout instead of sending them.
///
/// Used for local development and testing. Displays the email metadata
/// (from, to, subject) and content with ANSI formatting.
pub struct ConsoleProvider {
    /// Whether to print the full HTML body or just a truncated preview.
    pub show_full_html: bool,
    /// Maximum number of HTML characters to display when `show_full_html` is false.
    pub html_preview_length: usize,
}

impl ConsoleProvider {
    /// Create a new ConsoleProvider with default settings.
    pub fn new() -> Self {
        ConsoleProvider {
            show_full_html: false,
            html_preview_length: 500,
        }
    }

    /// Show the full HTML body in output.
    pub fn full_html(mut self) -> Self {
        self.show_full_html = true;
        self
    }
}

impl Default for ConsoleProvider {
    fn default() -> Self {
        ConsoleProvider::new()
    }
}

#[async_trait]
impl EmailProvider for ConsoleProvider {
    async fn send(&self, message: EmailMessage) -> Result<SendResult, EmailError> {
        let msg_id = CONSOLE_MSG_COUNTER.fetch_add(1, Ordering::Relaxed);
        let message_id = format!("console-{msg_id:08}");

        let separator = "─".repeat(60);
        let thin_sep = "─".repeat(40);

        println!();
        println!("\x1b[1;36m{separator}\x1b[0m");
        println!("\x1b[1;36m  📧 Ferrum Email — Console Provider\x1b[0m");
        println!("\x1b[1;36m{separator}\x1b[0m");
        println!();
        println!("  \x1b[1mFrom:\x1b[0m    {}", message.from);
        for to in &message.to {
            println!("  \x1b[1mTo:\x1b[0m      {to}");
        }
        for cc in &message.cc {
            println!("  \x1b[1mCc:\x1b[0m      {cc}");
        }
        for bcc in &message.bcc {
            println!("  \x1b[1mBcc:\x1b[0m     {bcc}");
        }
        if let Some(ref reply_to) = message.reply_to {
            println!("  \x1b[1mReply-To:\x1b[0m {reply_to}");
        }
        println!("  \x1b[1mSubject:\x1b[0m  {}", message.subject);
        println!("  \x1b[1mMsg ID:\x1b[0m   {message_id}");

        if !message.attachments.is_empty() {
            println!();
            println!("  \x1b[1mAttachments:\x1b[0m");
            for att in &message.attachments {
                println!(
                    "    - {} ({}, {} bytes)",
                    att.filename,
                    att.content_type,
                    att.content.len()
                );
            }
        }

        // Plain text version
        if let Some(ref text) = message.text {
            println!();
            println!("  \x1b[1;33m{thin_sep}\x1b[0m");
            println!("  \x1b[1;33m  Plain Text\x1b[0m");
            println!("  \x1b[1;33m{thin_sep}\x1b[0m");
            println!();
            for line in text.lines() {
                println!("  {line}");
            }
        }

        // HTML version
        println!();
        println!("  \x1b[1;32m{thin_sep}\x1b[0m");
        println!("  \x1b[1;32m  HTML\x1b[0m");
        println!("  \x1b[1;32m{thin_sep}\x1b[0m");
        println!();

        if self.show_full_html {
            for line in message.html.lines() {
                println!("  {line}");
            }
        } else {
            let preview = if message.html.len() > self.html_preview_length {
                format!(
                    "{}... \x1b[2m({} chars total)\x1b[0m",
                    &message.html[..self.html_preview_length],
                    message.html.len()
                )
            } else {
                message.html.clone()
            };
            for line in preview.lines() {
                println!("  {line}");
            }
        }

        println!();
        println!("\x1b[1;36m{separator}\x1b[0m");
        println!();

        Ok(SendResult {
            message_id,
            provider: "console".to_string(),
        })
    }
}
