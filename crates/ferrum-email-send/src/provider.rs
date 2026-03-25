//! The `EmailProvider` trait — the abstraction layer for email sending backends.

use async_trait::async_trait;

use crate::error::EmailError;
use crate::message::{EmailMessage, SendResult};

/// Trait for email sending providers.
///
/// Implement this trait to add support for a new email provider (e.g., Resend,
/// SendGrid, SMTP, etc.). Each provider handles serializing the `EmailMessage`
/// into the provider's specific API format and sending it.
///
/// # Example
///
/// ```rust,no_run
/// use async_trait::async_trait;
/// use ferrum_email_send::{EmailProvider, EmailMessage, SendResult, EmailError};
///
/// struct MyProvider;
///
/// #[async_trait]
/// impl EmailProvider for MyProvider {
///     async fn send(&self, message: EmailMessage) -> Result<SendResult, EmailError> {
///         // Send the email via your provider's API
///         Ok(SendResult {
///             message_id: "msg-123".to_string(),
///             provider: "my-provider".to_string(),
///         })
///     }
///
///     async fn send_batch(&self, messages: Vec<EmailMessage>) -> Result<Vec<SendResult>, EmailError> {
///         let mut results = Vec::new();
///         for msg in messages {
///             results.push(self.send(msg).await?);
///         }
///         Ok(results)
///     }
/// }
/// ```
#[async_trait]
pub trait EmailProvider: Send + Sync {
    /// Send a single email message.
    async fn send(&self, message: EmailMessage) -> Result<SendResult, EmailError>;

    /// Send multiple email messages in a batch.
    ///
    /// The default implementation sends each message sequentially.
    /// Providers may override this for bulk-send APIs.
    async fn send_batch(
        &self,
        messages: Vec<EmailMessage>,
    ) -> Result<Vec<SendResult>, EmailError> {
        let mut results = Vec::with_capacity(messages.len());
        for message in messages {
            results.push(self.send(message).await?);
        }
        Ok(results)
    }
}
