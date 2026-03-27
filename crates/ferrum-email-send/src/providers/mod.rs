//! Email provider implementations.
//!
//! Each provider implements the `EmailProvider` trait and handles sending
//! emails through a specific service or protocol.

pub mod console;
#[cfg(feature = "smtp")]
pub mod direct;
#[cfg(feature = "smtp")]
pub mod smtp;

pub use console::ConsoleProvider;
#[cfg(feature = "smtp")]
pub use direct::DirectMxProvider;
#[cfg(feature = "smtp")]
pub use smtp::SmtpProvider;
