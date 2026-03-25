//! NexusVault integration for secure credential storage.
//!
//! When the `vault` feature is enabled, Ferrum Email can store and retrieve
//! email provider credentials (API keys, SMTP passwords, etc.) from an
//! [AegisVault](https://github.com/AutomataNexus/Aegis-DB) instance.
//!
//! # Key Paths
//!
//! Credentials are stored under the `ferrum-email/` prefix:
//!
//! | Key | Value |
//! |-----|-------|
//! | `ferrum-email/provider/api-key` | Provider API key (Resend, SendGrid, etc.) |
//! | `ferrum-email/smtp/username` | SMTP username |
//! | `ferrum-email/smtp/password` | SMTP password |
//! | `ferrum-email/smtp/host` | SMTP server host |
//! | `ferrum-email/smtp/port` | SMTP server port |
//! | `ferrum-email/from/email` | Default sender email address |
//! | `ferrum-email/from/name` | Default sender display name |
//!
//! # Example
//!
//! ```rust,no_run
//! use ferrum_email_send::vault::VaultCredentialStore;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize vault
//! let vault = aegis_vault::AegisVault::init(aegis_vault::VaultConfig {
//!     data_dir: Some(std::path::PathBuf::from("/var/lib/ferrum/vault")),
//!     auto_unseal: true,
//!     passphrase: Some(std::env::var("VAULT_PASSPHRASE")?),
//!     ..Default::default()
//! }).await?;
//!
//! let store = VaultCredentialStore::new(Arc::new(vault));
//!
//! // Store credentials
//! store.set_api_key("re_abc123_my_resend_key")?;
//!
//! // Retrieve credentials
//! let api_key = store.get_api_key()?;
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;

use aegis_vault::AegisVault;

use crate::error::EmailError;
use crate::message::Mailbox;

/// The component name used for vault access control.
const VAULT_COMPONENT: &str = "ferrum-email";

/// Key prefix for all Ferrum Email secrets in the vault.
const KEY_PREFIX: &str = "ferrum-email";

/// Secure credential storage backed by NexusVault (AegisVault).
///
/// Provides typed get/set methods for email provider credentials,
/// all encrypted at rest with AES-256-GCM via the vault's master key.
pub struct VaultCredentialStore {
    vault: Arc<AegisVault>,
}

impl VaultCredentialStore {
    /// Create a new credential store backed by the given vault instance.
    pub fn new(vault: Arc<AegisVault>) -> Self {
        VaultCredentialStore { vault }
    }

    /// Check if the vault is sealed (credentials are inaccessible when sealed).
    pub fn is_sealed(&self) -> bool {
        self.vault.is_sealed()
    }

    // ── Provider API Key ──────────────────────────────────────────────

    /// Store an email provider API key (e.g., Resend, SendGrid).
    pub fn set_api_key(&self, api_key: &str) -> Result<(), EmailError> {
        self.set_secret("provider/api-key", api_key)
    }

    /// Retrieve the stored provider API key.
    pub fn get_api_key(&self) -> Result<String, EmailError> {
        self.get_secret("provider/api-key")
    }

    // ── SMTP Credentials ─────────────────────────────────────────────

    /// Store SMTP credentials (username, password, host, port).
    pub fn set_smtp_credentials(
        &self,
        username: &str,
        password: &str,
        host: &str,
        port: u16,
    ) -> Result<(), EmailError> {
        self.set_secret("smtp/username", username)?;
        self.set_secret("smtp/password", password)?;
        self.set_secret("smtp/host", host)?;
        self.set_secret("smtp/port", &port.to_string())?;
        Ok(())
    }

    /// Retrieve the stored SMTP username.
    pub fn get_smtp_username(&self) -> Result<String, EmailError> {
        self.get_secret("smtp/username")
    }

    /// Retrieve the stored SMTP password.
    pub fn get_smtp_password(&self) -> Result<String, EmailError> {
        self.get_secret("smtp/password")
    }

    /// Retrieve the stored SMTP host.
    pub fn get_smtp_host(&self) -> Result<String, EmailError> {
        self.get_secret("smtp/host")
    }

    /// Retrieve the stored SMTP port.
    pub fn get_smtp_port(&self) -> Result<u16, EmailError> {
        let port_str = self.get_secret("smtp/port")?;
        port_str
            .parse::<u16>()
            .map_err(|_| EmailError::Provider(format!("invalid SMTP port in vault: {port_str}")))
    }

    // ── Default Sender ───────────────────────────────────────────────

    /// Store the default sender email and display name.
    pub fn set_default_from(&self, email: &str, name: Option<&str>) -> Result<(), EmailError> {
        self.set_secret("from/email", email)?;
        if let Some(name) = name {
            self.set_secret("from/name", name)?;
        }
        Ok(())
    }

    /// Retrieve the default sender as a `Mailbox`.
    pub fn get_default_from(&self) -> Result<Mailbox, EmailError> {
        let email = self.get_secret("from/email")?;
        let name = self.get_secret("from/name").ok();
        Ok(match name {
            Some(name) => Mailbox::new(name, email),
            None => Mailbox::address(email),
        })
    }

    // ── Generic Secret Access ────────────────────────────────────────

    /// Store a custom secret under the ferrum-email prefix.
    pub fn set_custom(&self, key: &str, value: &str) -> Result<(), EmailError> {
        self.set_secret(&format!("custom/{key}"), value)
    }

    /// Retrieve a custom secret.
    pub fn get_custom(&self, key: &str) -> Result<String, EmailError> {
        self.get_secret(&format!("custom/{key}"))
    }

    /// List all stored credential keys.
    pub fn list_keys(&self) -> Result<Vec<String>, EmailError> {
        self.vault
            .list(&format!("{KEY_PREFIX}/"), VAULT_COMPONENT)
            .map_err(|e| EmailError::Provider(format!("vault list error: {e}")))
    }

    /// Delete a credential by key suffix (e.g., "provider/api-key").
    pub fn delete(&self, key_suffix: &str) -> Result<(), EmailError> {
        let full_key = format!("{KEY_PREFIX}/{key_suffix}");
        self.vault
            .delete(&full_key, VAULT_COMPONENT)
            .map_err(|e| EmailError::Provider(format!("vault delete error: {e}")))
    }

    // ── Internal ─────────────────────────────────────────────────────

    fn get_secret(&self, key_suffix: &str) -> Result<String, EmailError> {
        if self.vault.is_sealed() {
            return Err(EmailError::Provider(
                "vault is sealed — unseal before accessing credentials".to_string(),
            ));
        }
        let full_key = format!("{KEY_PREFIX}/{key_suffix}");
        self.vault
            .get(&full_key, VAULT_COMPONENT)
            .map_err(|e| EmailError::Provider(format!("vault error for {full_key}: {e}")))
    }

    fn set_secret(&self, key_suffix: &str, value: &str) -> Result<(), EmailError> {
        if self.vault.is_sealed() {
            return Err(EmailError::Provider(
                "vault is sealed — unseal before storing credentials".to_string(),
            ));
        }
        let full_key = format!("{KEY_PREFIX}/{key_suffix}");
        self.vault
            .set(&full_key, value, VAULT_COMPONENT)
            .map_err(|e| EmailError::Provider(format!("vault set error for {full_key}: {e}")))
    }
}
