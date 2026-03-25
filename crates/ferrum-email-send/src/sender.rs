//! The main `Sender` struct — the user-facing API for sending emails.

use ferrum_email_core::Component;
use ferrum_email_render::Renderer;

use crate::error::EmailError;
use crate::message::{EmailMessage, Mailbox, SendResult};
use crate::provider::EmailProvider;

/// The main entry point for sending emails.
///
/// Holds a provider, a default "from" address, and a renderer. Call `.send()`
/// with any `Component` to render and send it in one step.
///
/// # Example
///
/// ```rust,no_run
/// use ferrum_email_send::{Sender, providers::ConsoleProvider};
///
/// # async fn example() -> Result<(), ferrum_email_send::EmailError> {
/// let sender = Sender::new(ConsoleProvider::new(), "me@example.com");
/// // sender.send(my_email_component, "recipient@example.com").await?;
/// # Ok(())
/// # }
/// ```
pub struct Sender {
    provider: Box<dyn EmailProvider>,
    default_from: Mailbox,
    renderer: Renderer,
}

impl Sender {
    /// Create a new Sender with a provider and default "from" address.
    pub fn new(provider: impl EmailProvider + 'static, from: impl Into<Mailbox>) -> Self {
        Sender {
            provider: Box::new(provider),
            default_from: from.into(),
            renderer: Renderer::default(),
        }
    }

    /// Create a new Sender with a custom renderer.
    pub fn with_renderer(
        provider: impl EmailProvider + 'static,
        from: impl Into<Mailbox>,
        renderer: Renderer,
    ) -> Self {
        Sender {
            provider: Box::new(provider),
            default_from: from.into(),
            renderer,
        }
    }

    /// Send an email by rendering a component and sending it to a single recipient.
    ///
    /// The component's `subject()` method is used for the email subject line.
    /// The component is rendered to both HTML and plain text automatically.
    pub async fn send<C: Component>(
        &self,
        component: &C,
        to: impl Into<Mailbox>,
    ) -> Result<SendResult, EmailError> {
        let html = self.renderer.render_html(component)?;
        let text = self.renderer.render_text(component).ok();
        let subject = component.subject().unwrap_or("(no subject)").to_string();

        let message = EmailMessage {
            from: self.default_from.clone(),
            to: vec![to.into()],
            subject,
            html,
            text,
            ..Default::default()
        };

        self.provider.send(message).await
    }

    /// Send a pre-built EmailMessage directly via the provider.
    pub async fn send_message(&self, message: EmailMessage) -> Result<SendResult, EmailError> {
        self.provider.send(message).await
    }

    /// Send an email to multiple recipients.
    ///
    /// Each recipient receives an individual copy of the email.
    pub async fn send_batch<C: Component>(
        &self,
        component: &C,
        recipients: Vec<Mailbox>,
    ) -> Result<Vec<SendResult>, EmailError> {
        let html = self.renderer.render_html(component)?;
        let text = self.renderer.render_text(component).ok();
        let subject = component.subject().unwrap_or("(no subject)").to_string();

        let messages: Vec<EmailMessage> = recipients
            .into_iter()
            .map(|to| EmailMessage {
                from: self.default_from.clone(),
                to: vec![to],
                subject: subject.clone(),
                html: html.clone(),
                text: text.clone(),
                ..Default::default()
            })
            .collect();

        self.provider.send_batch(messages).await
    }

    /// Get a reference to the renderer.
    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }
}
