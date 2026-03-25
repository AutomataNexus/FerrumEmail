//! Error types for the email sending system.

/// Errors that can occur when sending emails.
#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    /// The email could not be rendered to HTML/text.
    #[error("render error: {0}")]
    Render(String),

    /// The email provider returned an error.
    #[error("provider error: {0}")]
    Provider(String),

    /// An email address could not be parsed.
    #[error("invalid address: {0}")]
    InvalidAddress(String),

    /// A required field was missing from the email message.
    #[error("missing required field: {0}")]
    MissingField(String),
}

impl From<ferrum_email_render::RenderError> for EmailError {
    fn from(err: ferrum_email_render::RenderError) -> Self {
        EmailError::Render(err.to_string())
    }
}
