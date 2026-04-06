//! Application state for the Ferrum Email TUI.

use crate::auth::{MailboxClient, Session};
use crate::templates;
use ferrum_email_render::Renderer;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Inbox,
    Compose,
    Outbox,
    Templates,
    Preview,
}

impl Tab {
    pub const ALL: &'static [Tab] = &[
        Tab::Inbox,
        Tab::Compose,
        Tab::Outbox,
        Tab::Templates,
        Tab::Preview,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Tab::Inbox => " Inbox ",
            Tab::Compose => " Compose ",
            Tab::Outbox => " Outbox ",
            Tab::Templates => " Templates ",
            Tab::Preview => " Preview ",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Preview,
    Sending,
    Compose,
    Reading,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComposeField {
    To,
    Subject,
    Body,
}

#[derive(Clone)]
pub struct MailItem {
    pub id: String,
    pub folder: String,
    pub from: String,
    pub to: String,
    pub subject: String,
    pub timestamp: String,
    pub status: String,
    pub preview: String,
    pub read: bool,
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

pub struct App {
    pub tab: Tab,
    pub tab_index: usize,
    pub mode: Mode,
    pub selected_template: usize,
    pub selected_inbox: usize,
    pub preview_html: String,
    pub preview_text: String,
    pub preview_scroll: u16,
    pub message: Option<(String, bool)>,
    pub compose_to: String,
    pub compose_subject: String,
    pub compose_body: String,
    pub compose_field: ComposeField,
    pub inbox: Vec<MailItem>,
    pub outbox: Vec<MailItem>,
    pub send_history: Vec<SendRecord>,
    pub reading_body: String,
    pub reading_subject: String,
    pub renderer: Renderer,
    pub session: Session,
    client: MailboxClient,
}

impl App {
    pub async fn new_with_session(
        session: &Session,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let client = MailboxClient::new(&session.token);
        let renderer = Renderer::default();

        let component = templates::render_template(0, &session.email);
        let preview_html = renderer.render_html(component.as_ref()).unwrap_or_default();
        let preview_text = renderer.render_text(component.as_ref()).unwrap_or_default();

        let mut app = App {
            tab: Tab::Inbox,
            tab_index: 0,
            mode: Mode::Normal,
            selected_template: 0,
            selected_inbox: 0,
            preview_html,
            preview_text,
            preview_scroll: 0,
            message: None,
            compose_to: String::new(),
            compose_subject: String::new(),
            compose_body: String::new(),
            compose_field: ComposeField::To,
            inbox: Vec::new(),
            outbox: Vec::new(),
            send_history: Vec::new(),
            reading_body: String::new(),
            reading_subject: String::new(),
            renderer,
            session: session.clone(),
            client,
        };

        // Load inbox
        let _ = app.refresh().await;
        Ok(app)
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
        match self.tab {
            Tab::Templates => {
                self.selected_template = (self.selected_template + 1) % templates::TEMPLATES.len();
            }
            Tab::Inbox => {
                if !self.inbox.is_empty() {
                    self.selected_inbox = (self.selected_inbox + 1) % self.inbox.len();
                }
            }
            _ => {}
        }
    }

    pub fn prev_item(&mut self) {
        match self.tab {
            Tab::Templates => {
                self.selected_template = if self.selected_template == 0 {
                    templates::TEMPLATES.len() - 1
                } else {
                    self.selected_template - 1
                };
            }
            Tab::Inbox => {
                if !self.inbox.is_empty() {
                    self.selected_inbox = if self.selected_inbox == 0 {
                        self.inbox.len() - 1
                    } else {
                        self.selected_inbox - 1
                    };
                }
            }
            _ => {}
        }
    }

    pub async fn select_item(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.tab {
            Tab::Inbox => {
                if let Some(msg) = self.inbox.get(self.selected_inbox) {
                    let folder = msg.folder.clone();
                    let id = msg.id.clone();
                    match self.client.get_message(&folder, &id) {
                        Ok(detail) => {
                            self.reading_subject = detail["meta"]["subject"]
                                .as_str().unwrap_or("").to_string();
                            self.reading_body = detail["text_body"]
                                .as_str()
                                .or_else(|| detail["html_body"].as_str())
                                .unwrap_or("(empty)")
                                .to_string();
                            // Strip HTML tags for display
                            if self.reading_body.contains('<') {
                                self.reading_body = self.reading_body
                                    .replace("<br>", "\n").replace("<br/>", "\n")
                                    .replace("</p>", "\n\n").replace("</div>", "\n");
                                let mut clean = String::new();
                                let mut in_tag = false;
                                for ch in self.reading_body.chars() {
                                    match ch {
                                        '<' => in_tag = true,
                                        '>' => { in_tag = false; }
                                        _ if !in_tag => clean.push(ch),
                                        _ => {}
                                    }
                                }
                                self.reading_body = clean.trim().to_string();
                            }
                            self.preview_scroll = 0;
                            self.mode = Mode::Reading;
                        }
                        Err(e) => self.message = Some((format!("Failed: {e}"), true)),
                    }
                }
            }
            Tab::Templates => {
                self.preview_selected();
            }
            _ => {}
        }
        Ok(())
    }

    pub fn preview_selected(&mut self) {
        let component = templates::render_template(self.selected_template, &self.session.email);
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
        let component = templates::render_template(self.selected_template, &self.session.email);

        let html = self.renderer.render_html(component.as_ref())?;
        let text = self.renderer.render_text(component.as_ref()).ok();
        let subject = component.subject().unwrap_or("(no subject)").to_string();

        match self.client.send_email(
            &[self.session.email.clone()],
            &subject,
            &html,
            text.as_deref(),
        ) {
            Ok(mid) => {
                self.send_history.push(SendRecord {
                    template: template_meta.name.to_string(),
                    to: self.session.email.clone(),
                    message_id: mid.clone(),
                    timestamp: "just now".into(),
                    success: true,
                });
                self.outbox.push(MailItem {
                    id: mid.clone(),
                    folder: "sent".into(),
                    from: self.session.email.clone(),
                    to: self.session.email.clone(),
                    subject: template_meta.subject.to_string(),
                    timestamp: "just now".into(),
                    status: "sent".into(),
                    preview: format!("Template: {}", template_meta.name),
                    read: true,
                });
                self.message = Some((format!("Sent (ID: {mid})"), false));
            }
            Err(e) => {
                self.message = Some((format!("Send failed: {e}"), true));
            }
        }
        self.mode = Mode::Normal;
        Ok(())
    }

    pub async fn refresh(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Fetch inbox
        match self.client.list_messages("inbox") {
            Ok(messages) => {
                self.inbox = messages.iter().filter_map(|m| {
                    Some(MailItem {
                        id: m["id"].as_str()?.to_string(),
                        folder: m["folder"].as_str().unwrap_or("inbox").to_string(),
                        from: m["from_display"].as_str()
                            .or_else(|| m["from"].as_str())
                            .unwrap_or("unknown").to_string(),
                        to: m["to"].as_array()
                            .and_then(|a| a.first())
                            .and_then(|v| v.as_str())
                            .unwrap_or("").to_string(),
                        subject: m["subject"].as_str().unwrap_or("(no subject)").to_string(),
                        timestamp: m["received_at"].as_str().unwrap_or("").to_string(),
                        status: if m["read"].as_bool().unwrap_or(false) { "read" } else { "unread" }.into(),
                        preview: m["preview"].as_str().unwrap_or("").to_string(),
                        read: m["read"].as_bool().unwrap_or(false),
                    })
                }).collect();
            }
            Err(e) => {
                self.message = Some((format!("Inbox fetch failed: {e}"), true));
            }
        }

        // Fetch sent
        match self.client.list_messages("sent") {
            Ok(messages) => {
                self.outbox = messages.iter().filter_map(|m| {
                    Some(MailItem {
                        id: m["id"].as_str()?.to_string(),
                        folder: "sent".into(),
                        from: self.session.email.clone(),
                        to: m["to"].as_array()
                            .and_then(|a| a.first())
                            .and_then(|v| v.as_str())
                            .unwrap_or("").to_string(),
                        subject: m["subject"].as_str().unwrap_or("(no subject)").to_string(),
                        timestamp: m["received_at"].as_str().unwrap_or("").to_string(),
                        status: "sent".into(),
                        preview: m["preview"].as_str().unwrap_or("").to_string(),
                        read: true,
                    })
                }).collect();
            }
            Err(_) => {}
        }

        self.message = Some((format!("{} messages", self.inbox.len()), false));
        Ok(())
    }

    pub fn dismiss_message(&mut self) {
        self.message = None;
        if self.mode == Mode::Preview || self.mode == Mode::Reading {
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
            ComposeField::To => { self.compose_to.pop(); }
            ComposeField::Subject => { self.compose_subject.pop(); }
            ComposeField::Body => { self.compose_body.pop(); }
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

        let html = format!(
            "<div style=\"font-family:-apple-system,sans-serif;color:#2D2A26;font-size:15px;line-height:1.6\">{}</div>",
            self.compose_body.replace('\n', "<br>")
        );

        let recipients: Vec<String> = self.compose_to
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        match self.client.send_email(&recipients, &self.compose_subject, &html, Some(&self.compose_body)) {
            Ok(mid) => {
                self.outbox.push(MailItem {
                    id: mid.clone(),
                    folder: "sent".into(),
                    from: self.session.email.clone(),
                    to: self.compose_to.clone(),
                    subject: self.compose_subject.clone(),
                    timestamp: "just now".into(),
                    status: "sent".into(),
                    preview: self.compose_body.chars().take(80).collect(),
                    read: true,
                });
                self.message = Some((format!("Sent to {} (ID: {mid})", self.compose_to), false));
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

    /// Delete selected inbox message (move to trash).
    pub fn delete_selected(&mut self) {
        if let Some(msg) = self.inbox.get(self.selected_inbox) {
            let _ = self.client.move_message(&msg.folder, &msg.id, "trash");
            self.inbox.remove(self.selected_inbox);
            if self.selected_inbox >= self.inbox.len() && self.selected_inbox > 0 {
                self.selected_inbox -= 1;
            }
            self.message = Some(("Moved to trash".to_string(), false));
        }
    }
}
