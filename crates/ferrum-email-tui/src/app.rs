//! Application state for the Ferrum Email TUI.

use std::path::PathBuf;
use std::sync::Arc;

use aegis_db_vault::{AegisVault, VaultConfig};
use ferrum_email_render::Renderer;
use ferrum_email_send::Sender;
use ferrum_email_send::providers::SmtpProvider;
use ferrum_email_send::vault::VaultCredentialStore;

use crate::templates;

const VAULT_DIR: &str = "/var/lib/ferrum-email/vault";

fn vault_passphrase() -> String {
    std::env::var("FERRUM_VAULT_PASSPHRASE")
        .unwrap_or_else(|_| "ferrum-email-vault-key".to_string())
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Inbox,
    Compose,
    Outbox,
    Templates,
    Preview,
    Vault,
    Send,
}

impl Tab {
    pub const ALL: &'static [Tab] = &[
        Tab::Inbox,
        Tab::Compose,
        Tab::Outbox,
        Tab::Templates,
        Tab::Preview,
        Tab::Vault,
        Tab::Send,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Tab::Inbox => " Inbox ",
            Tab::Compose => " Compose ",
            Tab::Outbox => " Outbox ",
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
    Compose,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComposeField {
    To,
    Subject,
    Body,
}

#[derive(Clone)]
pub struct MailItem {
    pub from: String,
    pub to: String,
    pub subject: String,
    pub timestamp: String,
    pub status: String,
    pub preview: String,
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
    pub compose_to: String,
    pub compose_subject: String,
    pub compose_body: String,
    pub compose_field: ComposeField,
    pub inbox: Vec<MailItem>,
    pub outbox: Vec<MailItem>,
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
#[allow(dead_code)]
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
            passphrase: Some(vault_passphrase()),
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
            tab: Tab::Inbox,
            tab_index: 0,
            mode: Mode::Normal,
            selected_template: 0,
            preview_html,
            preview_text,
            preview_scroll: 0,
            message: None,
            compose_to: smtp_user.clone(),
            compose_subject: String::new(),
            compose_body: String::new(),
            compose_field: ComposeField::To,
            inbox: vec![MailItem {
                from: "system@ferrum-mail.com".into(),
                to: smtp_user.clone(),
                subject: "Welcome to Ferrum Mail".into(),
                timestamp: "just now".into(),
                status: "unread".into(),
                preview: "Your account is ready. Start sending emails with your API key.".into(),
            }],
            outbox: Vec::new(),
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
            self.selected_template = (self.selected_template + 1) % templates::TEMPLATES.len();
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
        let component = templates::render_template(self.selected_template, &self.send_to);
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
        let component = templates::render_template(self.selected_template, &self.send_to);

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
                self.outbox.push(MailItem {
                    from: self.from.to_string(),
                    to: self.send_to.clone(),
                    subject: template_meta.subject.to_string(),
                    timestamp: "just now".into(),
                    status: "sent".into(),
                    preview: format!("Template: {}", template_meta.name),
                });
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

    // ── Compose mode ──

    pub fn enter_compose(&mut self) {
        self.mode = Mode::Compose;
        self.compose_field = ComposeField::To;
    }

    pub fn compose_next_field(&mut self) {
        self.compose_field = match self.compose_field {
            ComposeField::To => ComposeField::Subject,
            ComposeField::Subject => ComposeField::Body,
            ComposeField::Body => ComposeField::To,
        };
    }

    pub fn compose_prev_field(&mut self) {
        self.compose_field = match self.compose_field {
            ComposeField::To => ComposeField::Body,
            ComposeField::Subject => ComposeField::To,
            ComposeField::Body => ComposeField::Subject,
        };
    }

    pub fn compose_type_char(&mut self, ch: char) {
        match self.compose_field {
            ComposeField::To => self.compose_to.push(ch),
            ComposeField::Subject => self.compose_subject.push(ch),
            ComposeField::Body => self.compose_body.push(ch),
        }
    }

    pub fn compose_backspace(&mut self) {
        match self.compose_field {
            ComposeField::To => {
                self.compose_to.pop();
            }
            ComposeField::Subject => {
                self.compose_subject.pop();
            }
            ComposeField::Body => {
                self.compose_body.pop();
            }
        }
    }

    pub fn compose_newline(&mut self) {
        if self.compose_field == ComposeField::Body {
            self.compose_body.push('\n');
        }
    }

    pub async fn compose_send(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.compose_to.is_empty() || self.compose_subject.is_empty() {
            self.message = Some(("To and Subject are required".to_string(), true));
            return Ok(());
        }

        self.mode = Mode::Sending;

        // Build a simple HTML email from the compose fields
        let html_body = compose_to_html(&self.compose_subject, &self.compose_body);
        let text_body = self.compose_body.clone();

        let message = ferrum_email_send::EmailMessage {
            from: self.from.clone(),
            to: vec![self.compose_to.as_str().into()],
            subject: self.compose_subject.clone(),
            html: html_body,
            text: Some(text_body),
            ..Default::default()
        };

        let provider = SmtpProvider::builder()
            .host(&self.smtp_host)
            .port(self.smtp_port)
            .credentials(&self.smtp_user, &self.smtp_pass)
            .auth_login()
            .build()?;

        let sender = Sender::new(provider, self.from.clone());

        match sender.send_message(message).await {
            Ok(result) => {
                let record = SendRecord {
                    template: "Composed".to_string(),
                    to: self.compose_to.clone(),
                    message_id: result.message_id.clone(),
                    timestamp: self.compose_subject.clone(),
                    success: true,
                };
                self.send_history.push(record);
                self.outbox.push(MailItem {
                    from: self.from.to_string(),
                    to: self.compose_to.clone(),
                    subject: self.compose_subject.clone(),
                    timestamp: "just now".into(),
                    status: "sent".into(),
                    preview: self.compose_body.chars().take(80).collect(),
                });
                self.message = Some((
                    format!("Sent to {} (ID: {})", self.compose_to, result.message_id),
                    false,
                ));
                // Clear compose fields after success
                self.compose_subject.clear();
                self.compose_body.clear();
            }
            Err(e) => {
                self.message = Some((format!("Send failed: {e}"), true));
            }
        }
        self.mode = Mode::Normal;
        Ok(())
    }
}

/// Convert compose text to branded HTML email.
fn compose_to_html(subject: &str, body: &str) -> String {
    use ferrum_email_components::*;

    const FERRUM_LOGO: &str = "https://raw.githubusercontent.com/AutomataNexus/FerrumEmail/master/assets/FerrumEmail_logo.PNG";
    const NEXUS_LOGO: &str = "https://raw.githubusercontent.com/AutomataNexus/FerrumEmail/master/assets/AutomataNexus_Logo.PNG";

    let body_paragraphs: Vec<Node> = body
        .split('\n')
        .filter(|l| !l.is_empty())
        .map(|line| {
            Text::new(line)
                .color(Color::hex("4A4540"))
                .font_size(Px(15))
                .line_height(1.6)
                .into_node()
        })
        .collect();

    let mut section = Section::new()
        .padding(Spacing::new(Px(0), Px(40), Px(32), Px(40)))
        .background(Color::hex("FFFEFA"))
        .child_node(
            Heading::h2(subject)
                .color(Color::hex("2D2A26"))
                .margin(Spacing::new(Px(0), Px(0), Px(16), Px(0)))
                .into_node(),
        );

    for p in body_paragraphs {
        section = section.child_node(p);
    }

    let footer = Section::new()
        .padding(Spacing::new(Px(28), Px(40), Px(32), Px(40)))
        .background(Color::hex("FAF8F5"))
        .text_align(TextAlign::Center)
        .child_node(Image::new(NEXUS_LOGO, "AutomataNexus", Px(140)).into_node())
        .child_node(Spacer::new(Px(14)).into_node())
        .child_node(
            Text::new("Secured by NexusVault")
                .color(Color::hex("8B6F5E"))
                .font_size(Px(13))
                .font_weight(FontWeight::SemiBold)
                .text_align(TextAlign::Center)
                .margin(Spacing::new(Px(0), Px(0), Px(4), Px(0)))
                .into_node(),
        )
        .child_node(Hr::new().color(Color::hex("E8DDD4")).into_node())
        .child_node(Spacer::new(Px(12)).into_node())
        .child_node(
            Text::new("\u{00A9} 2026 AutomataNexus LLC. All rights reserved.")
                .color(Color::hex("A8998C"))
                .font_size(Px(11))
                .text_align(TextAlign::Center)
                .into_node(),
        );

    let email = Html::new().child(Head::new().title(subject)).child(
        Body::new().background(Color::hex("FAFAF8")).child(
            Container::new()
                .max_width(Px(600))
                .padding(Spacing::xy(Px(20), Px(0)))
                .child_node(
                    Section::new()
                        .padding(Spacing::new(Px(40), Px(40), Px(20), Px(40)))
                        .background(Color::hex("FFFEFA"))
                        .text_align(TextAlign::Center)
                        .child_node(Image::new(FERRUM_LOGO, "Ferrum Email", Px(280)).into_node())
                        .into_node(),
                )
                .child_node(section.into_node())
                .child_node(footer.into_node()),
        ),
    );

    let renderer = ferrum_email_render::Renderer::default();
    renderer.render_html(&email).unwrap_or_default()
}
