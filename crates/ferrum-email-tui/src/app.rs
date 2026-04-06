//! Application state for the Ferrum Email TUI — SDK developer tool.
//! Connects to the Ferrum Mail SaaS API for sending transactional emails.

use crate::auth::{SaasClient, Session};
use crate::templates;
use ferrum_email_render::Renderer;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Dashboard,
    Compose,
    Templates,
    Preview,
    SendHistory,
}

impl Tab {
    pub const ALL: &'static [Tab] = &[
        Tab::Dashboard,
        Tab::Compose,
        Tab::Templates,
        Tab::Preview,
        Tab::SendHistory,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Tab::Dashboard => " Dashboard ",
            Tab::Compose => " Compose ",
            Tab::Templates => " Templates ",
            Tab::Preview => " Preview ",
            Tab::SendHistory => " Send History ",
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
    pub preview_html: String,
    pub preview_text: String,
    pub preview_scroll: u16,
    pub message: Option<(String, bool)>,
    pub compose_to: String,
    pub compose_subject: String,
    pub compose_body: String,
    pub compose_field: ComposeField,
    pub send_history: Vec<SendRecord>,
    pub dashboard_stats: DashboardStats,
    pub api_keys: Vec<String>,
    pub renderer: Renderer,
    pub session: Session,
    client: SaasClient,
}

#[derive(Default)]
pub struct DashboardStats {
    pub emails_sent: u64,
    pub emails_today: u64,
    pub plan: String,
    pub quota: String,
}

impl App {
    pub async fn new_with_session(session: &Session) -> Result<Self, Box<dyn std::error::Error>> {
        let client = SaasClient::new(&session.token);
        let renderer = Renderer::default();

        let component = templates::render_template(0, &session.email);
        let preview_html = renderer.render_html(component.as_ref()).unwrap_or_default();
        let preview_text = renderer.render_text(component.as_ref()).unwrap_or_default();

        let mut app = App {
            tab: Tab::Dashboard,
            tab_index: 0,
            mode: Mode::Normal,
            selected_template: 0,
            preview_html,
            preview_text,
            preview_scroll: 0,
            message: None,
            compose_to: String::new(),
            compose_subject: String::new(),
            compose_body: String::new(),
            compose_field: ComposeField::To,
            send_history: Vec::new(),
            dashboard_stats: DashboardStats::default(),
            api_keys: Vec::new(),
            renderer,
            session: session.clone(),
            client,
        };

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

        // Send via SaaS API
        match self.client.send_email(
            &self.session.email,
            &subject,
            &html,
            text.as_deref(),
            self.session.api_key.as_deref(),
        ) {
            Ok(mid) => {
                self.send_history.push(SendRecord {
                    template: template_meta.name.to_string(),
                    to: self.session.email.clone(),
                    message_id: mid.clone(),
                    timestamp: "just now".into(),
                    success: true,
                });
                self.message = Some((
                    format!(
                        "Sent \"{}\" to {} (ID: {mid})",
                        template_meta.name, self.session.email
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
        // Fetch dashboard stats
        if let Ok(dash) = self.client.dashboard() {
            self.dashboard_stats = DashboardStats {
                emails_sent: dash["total_sends"].as_u64().unwrap_or(0),
                emails_today: dash["sends_today"].as_u64().unwrap_or(0),
                plan: dash["plan"].as_str().unwrap_or("Free").to_string(),
                quota: dash["quota"]
                    .as_str()
                    .or_else(|| dash["monthly_quota"].as_str())
                    .unwrap_or("—")
                    .to_string(),
            };
        }

        // Fetch API keys
        if let Ok(keys) = self.client.list_keys() {
            self.api_keys = keys
                .iter()
                .filter_map(|k| k["prefix"].as_str().map(|p| format!("{}...", p)))
                .collect();
        }

        // Fetch send history
        if let Ok(sends) = self.client.send_history() {
            self.send_history = sends
                .iter()
                .map(|s| SendRecord {
                    template: s["subject"].as_str().unwrap_or("email").to_string(),
                    to: s["to"].as_str().unwrap_or("?").to_string(),
                    message_id: s["message_id"].as_str().unwrap_or("?").to_string(),
                    timestamp: s["sent_at"].as_str().unwrap_or("").to_string(),
                    success: s["status"].as_str() == Some("sent"),
                })
                .collect();
        }

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

        let html = compose_to_html(&self.compose_subject, &self.compose_body);
        let text = self.compose_body.clone();

        match self.client.send_email(
            &self.compose_to,
            &self.compose_subject,
            &html,
            Some(&text),
            self.session.api_key.as_deref(),
        ) {
            Ok(mid) => {
                self.send_history.push(SendRecord {
                    template: "Composed".to_string(),
                    to: self.compose_to.clone(),
                    message_id: mid.clone(),
                    timestamp: "just now".into(),
                    success: true,
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
}

/// Convert compose text to branded HTML email.
fn compose_to_html(subject: &str, body: &str) -> String {
    use ferrum_email_components::*;

    const FERRUM_LOGO: &str = "https://raw.githubusercontent.com/AutomataNexus/FerrumEmail/master/assets/FerrumEmail_logo.PNG";

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
                .child_node(section.into_node()),
        ),
    );

    let renderer = ferrum_email_render::Renderer::default();
    renderer.render_html(&email).unwrap_or_default()
}
