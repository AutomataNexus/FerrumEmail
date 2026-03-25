//! UI rendering for the Ferrum Email TUI dashboard.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};

use crate::app::{App, Mode, Tab};
use crate::templates::TEMPLATES;
use crate::theme;

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();

    // Background fill
    let bg_block = Block::default().style(Style::default().bg(theme::BG));
    f.render_widget(bg_block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header + tabs
            Constraint::Min(10),  // main content
            Constraint::Length(3), // status bar
        ])
        .split(size);

    draw_header(f, app, chunks[0]);
    draw_main(f, app, chunks[1]);
    draw_statusbar(f, app, chunks[2]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(22), Constraint::Min(20)])
        .split(area);

    // Logo / title
    let title = Paragraph::new(Line::from(vec![
        Span::styled(" Ferrum", theme::title()),
        Span::styled(" Email ", theme::title_secondary()),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(theme::border_style())
            .style(Style::default().bg(theme::BG)),
    );
    f.render_widget(title, chunks[0]);

    // Tabs
    let tab_titles: Vec<Line> = Tab::ALL
        .iter()
        .enumerate()
        .map(|(i, t)| {
            if i == app.tab_index {
                Line::from(Span::styled(t.label(), theme::tab_active()))
            } else {
                Line::from(Span::styled(t.label(), theme::tab_inactive()))
            }
        })
        .collect();

    let tabs = Tabs::new(tab_titles)
        .select(app.tab_index)
        .divider(Span::styled(" | ", theme::text_dim()))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(theme::border_style())
                .style(Style::default().bg(theme::BG)),
        );
    f.render_widget(tabs, chunks[1]);
}

fn draw_main(f: &mut Frame, app: &App, area: Rect) {
    match app.mode {
        Mode::Preview => draw_preview(f, app, area),
        _ => match app.tab {
            Tab::Templates => draw_templates(f, app, area),
            Tab::Preview => draw_preview(f, app, area),
            Tab::Vault => draw_vault(f, app, area),
            Tab::Send => draw_send_history(f, app, area),
        },
    }
}

fn draw_templates(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);

    // Template list
    let items: Vec<ListItem> = TEMPLATES
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let style = if i == app.selected_template {
                theme::highlight()
            } else {
                theme::text_normal()
            };
            let marker = if i == app.selected_template {
                " > "
            } else {
                "   "
            };
            ListItem::new(Line::from(vec![
                Span::styled(marker, style),
                Span::styled(t.name, style),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(Span::styled(" Templates ", theme::label()))
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .style(Style::default().bg(theme::CARD_BG)),
    );
    f.render_widget(list, chunks[0]);

    // Template details
    let tmpl = &TEMPLATES[app.selected_template];
    let details = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Name:     ", theme::label()),
            Span::styled(tmpl.name, theme::text_normal()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Subject:  ", theme::label()),
            Span::styled(tmpl.subject, theme::text_normal()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Desc:     ", theme::label()),
            Span::styled(tmpl.description, theme::text_muted()),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Send to:  ", theme::label()),
            Span::styled(&app.send_to, theme::text_normal()),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [Enter]", theme::keybind()),
            Span::styled(" Preview  ", theme::keybind_desc()),
            Span::styled("[s]", theme::keybind()),
            Span::styled(" Send  ", theme::keybind_desc()),
        ]),
    ])
    .block(
        Block::default()
            .title(Span::styled(" Details ", theme::label()))
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .style(Style::default().bg(theme::CARD_BG)),
    );
    f.render_widget(details, chunks[1]);
}

fn draw_preview(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // HTML preview
    let html_lines: Vec<Line> = app
        .preview_html
        .lines()
        .skip(app.preview_scroll as usize)
        .map(|l| Line::from(Span::styled(l, theme::text_muted())))
        .collect();

    let html_view = Paragraph::new(html_lines)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .title(Span::styled(" HTML Output ", theme::label()))
                .borders(Borders::ALL)
                .border_style(theme::border_style())
                .style(Style::default().bg(theme::CARD_BG)),
        );
    f.render_widget(html_view, chunks[0]);

    // Plain text preview
    let text_lines: Vec<Line> = app
        .preview_text
        .lines()
        .skip(app.preview_scroll as usize)
        .map(|l| Line::from(Span::styled(l, theme::text_normal())))
        .collect();

    let text_view = Paragraph::new(text_lines)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .title(Span::styled(" Plain Text ", theme::label()))
                .borders(Borders::ALL)
                .border_style(theme::border_style())
                .style(Style::default().bg(theme::CARD_BG)),
        );
    f.render_widget(text_view, chunks[1]);
}

fn draw_vault(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(5)])
        .split(area);

    // Vault status
    let status = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Status:  ", theme::label()),
            Span::styled(&app.vault_status, theme::status_ok()),
        ]),
        Line::from(vec![
            Span::styled("  Path:    ", theme::label()),
            Span::styled("/var/lib/ferrum-email/vault", theme::text_muted()),
        ]),
    ])
    .block(
        Block::default()
            .title(Span::styled(" NexusVault ", theme::label()))
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .style(Style::default().bg(theme::CARD_BG)),
    );
    f.render_widget(status, chunks[0]);

    // Vault keys
    let items: Vec<ListItem> = app
        .vault_keys
        .iter()
        .map(|k| {
            let display = if k.contains("password") || k.contains("api-key") {
                format!("  {} = ********", k)
            } else {
                format!("  {}", k)
            };
            ListItem::new(Line::from(Span::styled(display, theme::text_normal())))
        })
        .collect();

    let keys_list = List::new(items).block(
        Block::default()
            .title(Span::styled(
                " Stored Credentials (AES-256-GCM) ",
                theme::label(),
            ))
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .style(Style::default().bg(theme::CARD_BG)),
    );
    f.render_widget(keys_list, chunks[1]);
}

fn draw_send_history(f: &mut Frame, app: &App, area: Rect) {
    if app.send_history.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "  No emails sent yet. Select a template and press [s] to send.",
                theme::text_dim(),
            )),
        ])
        .block(
            Block::default()
                .title(Span::styled(" Send History ", theme::label()))
                .borders(Borders::ALL)
                .border_style(theme::border_style())
                .style(Style::default().bg(theme::CARD_BG)),
        );
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .send_history
        .iter()
        .rev()
        .map(|r| {
            let status_style = if r.success {
                theme::status_ok()
            } else {
                theme::status_err()
            };
            let icon = if r.success { "OK" } else { "FAIL" };
            ListItem::new(Line::from(vec![
                Span::styled(format!("  [{icon}] "), status_style),
                Span::styled(&r.template, theme::text_normal()),
                Span::styled(" -> ", theme::text_dim()),
                Span::styled(&r.to, theme::text_muted()),
                Span::styled(format!("  ({})", r.message_id), theme::text_dim()),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(Span::styled(" Send History ", theme::label()))
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .style(Style::default().bg(theme::CARD_BG)),
    );
    f.render_widget(list, area);
}

fn draw_statusbar(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(40), Constraint::Length(40)])
        .split(area);

    // Message or keybindings
    let left_content = if let Some((ref msg, is_error)) = app.message {
        let style = if is_error {
            theme::status_err()
        } else {
            theme::status_ok()
        };
        Line::from(vec![Span::styled(format!("  {msg}"), style)])
    } else {
        let mode_label = match app.mode {
            Mode::Normal => "NORMAL",
            Mode::Preview => "PREVIEW",
            Mode::Sending => "SENDING...",
        };
        Line::from(vec![
            Span::styled(format!("  [{mode_label}] "), theme::label()),
            Span::styled("[Tab]", theme::keybind()),
            Span::styled(" Switch  ", theme::keybind_desc()),
            Span::styled("[j/k]", theme::keybind()),
            Span::styled(" Navigate  ", theme::keybind_desc()),
            Span::styled("[s]", theme::keybind()),
            Span::styled(" Send  ", theme::keybind_desc()),
            Span::styled("[q]", theme::keybind()),
            Span::styled(" Quit", theme::keybind_desc()),
        ])
    };

    let left = Paragraph::new(left_content).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(theme::border_style())
            .style(Style::default().bg(theme::SUBTLE_BG)),
    );
    f.render_widget(left, chunks[0]);

    // Branding
    let right = Paragraph::new(Line::from(vec![
        Span::styled("Secured by ", theme::text_dim()),
        Span::styled("NexusVault ", Style::default().fg(theme::TEAL).add_modifier(Modifier::BOLD)),
        Span::styled(" ", theme::text_dim()),
    ]))
    .alignment(Alignment::Right)
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(theme::border_style())
            .style(Style::default().bg(theme::SUBTLE_BG)),
    );
    f.render_widget(right, chunks[1]);
}
