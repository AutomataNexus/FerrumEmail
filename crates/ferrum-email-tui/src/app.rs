//! Application state for the Ferrum Email TUI.

use std::path::PathBuf;
use std::sync::Arc;

use aegis_db_vault::{AegisVault, VaultConfig};
use ferrum_email_render::Renderer;
use ferrum_email_send::providers::SmtpProvider;
use ferrum_email_send::vault::VaultCredentialStore;
use ferrum_email_send::Sender;

use ferrum_email_core::Component;

use crate::templates;

const VAULT_DIR: &str = "/var/lib/ferrum-email/vault";
const VAULT_PASSPHRASE: &str = "ferrum-email-vault-key";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Templates,
    Preview,
    Vault,
    Send,
}

impl Tab {
    pub const ALL: &'static [Tab] = &[Tab::Templates, Tab::Preview, Tab::Vault, Tab::Send];

    pub fn label(&self) -> &'static str {
        match self {
            Tab::Templates => " Templates ",
            Tab::Preview => " Preview ",
            Tab::Vault => " Vault ",
            Tab::Send => " Send ",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Preview,
    Sending,
}

pub struct App {
    pub tab: Tab,
    pub tab_index: usize,
    pub mode: Mode,
    pub selected_template: usize,
    pub preview_html: String,
    pub preview_text: String,
    pub preview_scroll: u16,
    pub message: Option<(String, bool)>, // (text, is_error)
    pub vault_keys: Vec<String>,
    pub vault_status: String,
    pub send_to: String,
    pub send_history: Vec<SendRecord>,
    pub renderer: Renderer,
    store: VaultCredentialStore,
    smtp_host: String,
    smtp_port: u16,
    smtp_user: String,
    smtp_pass: String,
    from: ferrum_email_send::Mailbox,
}

#[derive(Clone)]
pub struct SendRecord {
    pub template: String,
    pub to: String,
    pub message_id: String,
    pub timestamp: String,
    pub success: bool,
}

impl App {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = VaultConfig {
            data_dir: Some(PathBuf::from(VAULT_DIR)),
            auto_unseal: true,
            passphrase: Some(VAULT_PASSPHRASE.to_string()),
            ..Default::default()
        };
        let vault = AegisVault::init(config).await?;
        let vault = Arc::new(vault);
        let store = VaultCredentialStore::new(vault.clone());

        let smtp_user = store.get_smtp_username().unwrap_or_default();
        let smtp_pass = store.get_smtp_password().unwrap_or_default();
        let smtp_host = store.get_smtp_host().unwrap_or_default();
        let smtp_port = store.get_smtp_port().unwrap_or(587);
        let from = store
            .get_default_from()
            .unwrap_or_else(|_| ferrum_email_send::Mailbox::address("noreply@example.com"));
        let vault_keys = store.list_keys().unwrap_or_default();

        let vault_status = if vault.is_sealed() {
            "Sealed".to_string()
        } else {
            format!("Unsealed ({} keys)", vault_keys.len())
        };

        let renderer = Renderer::default();

        // Pre-render first template
        let component = templates::render_template(0, &smtp_user);
        let preview_html = renderer.render_html(component.as_ref()).unwrap_or_default();
        let preview_text = renderer.render_text(component.as_ref()).unwrap_or_default();

        Ok(App {
            tab: Tab::Templates,
            tab_index: 0,
            mode: Mode::Normal,
            selected_template: 0,
            preview_html,
            preview_text,
            preview_scroll: 0,
            message: None,
            vault_keys,
            vault_status,
            send_to: smtp_user.clone(),
            send_history: Vec::new(),
            renderer,
            store,
            smtp_host,
            smtp_port,
            smtp_user,
            smtp_pass,
            from,
        })
    }

    pub fn next_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % Tab::ALL.len();
        self.tab = Tab::ALL[self.tab_index];
    }

    pub fn prev_tab(&mut self) {
        self.tab_index = if self.tab_index == 0 {
            Tab::ALL.len() - 1
        } else {
            self.tab_index - 1
        };
        self.tab = Tab::ALL[self.tab_index];
    }

    pub fn next_item(&mut self) {
        if self.tab == Tab::Templates {
            self.selected_template =
                (self.selected_template + 1) % templates::TEMPLATES.len();
        }
    }

    pub fn prev_item(&mut self) {
        if self.tab == Tab::Templates {
            self.selected_template = if self.selected_template == 0 {
                templates::TEMPLATES.len() - 1
            } else {
                self.selected_template - 1
            };
        }
    }

    pub async fn select_item(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.preview_selected();
        Ok(())
    }

    pub fn preview_selected(&mut self) {
        let component =
            templates::render_template(self.selected_template, &self.send_to);
        self.preview_html = self
            .renderer
            .render_html(component.as_ref())
            .unwrap_or_else(|e| format!("Render error: {e}"));
        self.preview_text = self
            .renderer
            .render_text(component.as_ref())
            .unwrap_or_else(|e| format!("Render error: {e}"));
        self.preview_scroll = 0;
        self.mode = Mode::Preview;
    }

    pub async fn send_selected(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.mode = Mode::Sending;
        let template_meta = &templates::TEMPLATES[self.selected_template];
        let component =
            templates::render_template(self.selected_template, &self.send_to);

        let provider = SmtpProvider::builder()
            .host(&self.smtp_host)
            .port(self.smtp_port)
            .credentials(&self.smtp_user, &self.smtp_pass)
            .auth_login()
            .build()?;

        let sender = Sender::new(provider, self.from.clone());

        // Build the message manually since we have a dyn Component
        let html = self.renderer.render_html(component.as_ref())?;
        let text = self.renderer.render_text(component.as_ref()).ok();
        let subject = component.subject().unwrap_or("(no subject)").to_string();

        let message = ferrum_email_send::EmailMessage {
            from: self.from.clone(),
            to: vec![self.send_to.as_str().into()],
            subject,
            html,
            text,
            ..Default::default()
        };

        match sender.send_message(message).await {
            Ok(result) => {
                let record = SendRecord {
                    template: template_meta.name.to_string(),
                    to: self.send_to.clone(),
                    message_id: result.message_id.clone(),
                    timestamp: template_meta.subject.to_string(),
                    success: true,
                };
                self.send_history.push(record);
                self.message = Some((
                    format!(
                        "Sent \"{}\" to {} (ID: {})",
                        template_meta.name, self.send_to, result.message_id
                    ),
                    false,
                ));
            }
            Err(e) => {
                self.message = Some((format!("Send failed: {e}"), true));
            }
        }
        self.mode = Mode::Normal;
        Ok(())
    }

    pub async fn refresh(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.vault_keys = self.store.list_keys().unwrap_or_default();
        self.message = Some(("Refreshed".to_string(), false));
        Ok(())
    }

    pub fn dismiss_message(&mut self) {
        self.message = None;
        if self.mode == Mode::Preview {
            self.mode = Mode::Normal;
        }
    }

    pub fn scroll_down(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(1);
    }
}
