//! Email provider implementations.
//!
//! Each provider implements the `EmailProvider` trait and handles sending
//! emails through a specific service or protocol.

pub mod console;

pub use console::ConsoleProvider;
