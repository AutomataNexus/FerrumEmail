mod app;
mod auth;
mod templates;
mod theme;
mod ui;

use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use std::io;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Try auto-login from saved session
    let session = auth::load_session();

    let result = if let Some(session) = session {
        // Already logged in — go straight to dashboard
        let mut app = App::new_with_session(&session).await?;
        run_app(&mut terminal, &mut app).await
    } else {
        // Show login screen
        let session = run_login(&mut terminal)?;
        let mut app = App::new_with_session(&session).await?;
        run_app(&mut terminal, &mut app).await
    };

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {e}");
    }

    Ok(())
}

fn run_login<B: Backend>(
    terminal: &mut Terminal<B>,
) -> Result<auth::Session, Box<dyn std::error::Error>> {
    let mut email = String::new();
    let mut password = String::new();
    let mut focused = 0; // 0 = email, 1 = password
    let mut error: Option<String> = None;
    let mut logging_in = false;

    loop {
        terminal.draw(|f| {
            let size = f.area();
            let bg = ratatui::widgets::Block::default().style(Style::default().bg(theme::BG));
            f.render_widget(bg, size);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Length(14),
                    Constraint::Percentage(25),
                ])
                .horizontal_margin(size.width.saturating_sub(50) / 2)
                .split(size);

            let inner = chunks[1];
            let block = ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .border_style(Style::default().fg(theme::TERRACOTTA))
                .title(" Ferrum Mail — Sign In ")
                .title_style(Style::default().fg(theme::TERRACOTTA).bold());

            let inner_area = block.inner(inner);
            f.render_widget(block, inner);

            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // spacing
                    Constraint::Length(1), // email label
                    Constraint::Length(1), // email input
                    Constraint::Length(1), // spacing
                    Constraint::Length(1), // password label
                    Constraint::Length(1), // password input
                    Constraint::Length(1), // spacing
                    Constraint::Length(1), // button / error
                    Constraint::Length(1), // status
                ])
                .split(inner_area);

            let email_label = ratatui::text::Line::from(vec![Span::styled(
                "  Email: ",
                Style::default().fg(theme::LABEL),
            )]);
            f.render_widget(ratatui::widgets::Paragraph::new(email_label), rows[1]);

            let email_style = if focused == 0 {
                Style::default().fg(theme::TERRACOTTA).underlined()
            } else {
                Style::default().fg(theme::TEXT)
            };
            let email_line = ratatui::text::Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    if email.is_empty() {
                        "you@example.com"
                    } else {
                        &email
                    },
                    if email.is_empty() {
                        Style::default().fg(theme::TEXT_DIM)
                    } else {
                        email_style
                    },
                ),
            ]);
            f.render_widget(ratatui::widgets::Paragraph::new(email_line), rows[2]);

            let pw_label = ratatui::text::Line::from(vec![Span::styled(
                "  Password: ",
                Style::default().fg(theme::LABEL),
            )]);
            f.render_widget(ratatui::widgets::Paragraph::new(pw_label), rows[4]);

            let pw_style = if focused == 1 {
                Style::default().fg(theme::TERRACOTTA).underlined()
            } else {
                Style::default().fg(theme::TEXT)
            };
            let masked = "*".repeat(password.len());
            let pw_line = ratatui::text::Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    if password.is_empty() {
                        "••••••••"
                    } else {
                        &masked
                    },
                    if password.is_empty() {
                        Style::default().fg(theme::TEXT_DIM)
                    } else {
                        pw_style
                    },
                ),
            ]);
            f.render_widget(ratatui::widgets::Paragraph::new(pw_line), rows[5]);

            if let Some(ref err) = error {
                let err_line = ratatui::text::Line::from(vec![Span::styled(
                    format!("  {err}"),
                    Style::default().fg(theme::ERROR),
                )]);
                f.render_widget(ratatui::widgets::Paragraph::new(err_line), rows[7]);
            } else if logging_in {
                let status = ratatui::text::Line::from(vec![Span::styled(
                    "  Signing in...",
                    Style::default().fg(theme::TEAL),
                )]);
                f.render_widget(ratatui::widgets::Paragraph::new(status), rows[7]);
            } else {
                let hint = ratatui::text::Line::from(vec![Span::styled(
                    "  [Tab] switch  [Enter] sign in  [Esc] quit",
                    Style::default().fg(theme::TEXT_DIM),
                )]);
                f.render_widget(ratatui::widgets::Paragraph::new(hint), rows[7]);
            }

            let footer = ratatui::text::Line::from(vec![Span::styled(
                "  Sign up at ferrum-mail.com",
                Style::default().fg(theme::TEXT_DIM),
            )]);
            f.render_widget(ratatui::widgets::Paragraph::new(footer), rows[8]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            error = None;
            match key.code {
                KeyCode::Esc => {
                    return Err("Login cancelled".into());
                }
                KeyCode::Tab | KeyCode::BackTab => {
                    focused = 1 - focused;
                }
                KeyCode::Enter => {
                    if email.is_empty() || password.is_empty() {
                        error = Some("Enter email and password".into());
                        continue;
                    }
                    #[allow(unused_assignments)]
                    {
                        logging_in = true;
                    }

                    match auth::login(&email, &password) {
                        Ok(session) => return Ok(session),
                        Err(e) => {
                            error = Some(e);
                            logging_in = false;
                        }
                    }
                }
                KeyCode::Char(ch) => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) && ch == 'c' {
                        return Err("Login cancelled".into());
                    }
                    if focused == 0 {
                        email.push(ch);
                    } else {
                        password.push(ch);
                    }
                }
                KeyCode::Backspace => {
                    if focused == 0 {
                        email.pop();
                    } else {
                        password.pop();
                    }
                }
                _ => {}
            }
        }
    }
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                return Ok(());
            }

            match app.mode {
                app::Mode::Normal => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Tab | KeyCode::Char('l') | KeyCode::Right => app.next_tab(),
                    KeyCode::BackTab | KeyCode::Char('h') | KeyCode::Left => app.prev_tab(),
                    KeyCode::Char('j') | KeyCode::Down => app.next_item(),
                    KeyCode::Char('k') | KeyCode::Up => app.prev_item(),
                    KeyCode::Enter => {
                        if app.tab == app::Tab::Compose {
                            app.enter_compose();
                        } else {
                            app.select_item().await?;
                        }
                    }
                    KeyCode::Char('s') => app.send_selected().await?,
                    KeyCode::Char('p') => app.preview_selected(),
                    KeyCode::Char('r') => app.refresh().await?,
                    KeyCode::Esc => app.dismiss_message(),
                    _ => {}
                },
                app::Mode::Compose => match key.code {
                    KeyCode::Esc => app.mode = app::Mode::Normal,
                    KeyCode::Tab => app.compose_next_field(),
                    KeyCode::BackTab => app.compose_prev_field(),
                    KeyCode::Char(ch) => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) && ch == 's' {
                            app.compose_send().await?;
                        } else {
                            app.compose_type_char(ch);
                        }
                    }
                    KeyCode::Backspace => app.compose_backspace(),
                    KeyCode::Enter => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            app.compose_send().await?;
                        } else {
                            app.compose_newline();
                        }
                    }
                    _ => {}
                },
                app::Mode::Preview => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => app.mode = app::Mode::Normal,
                    KeyCode::Char('j') | KeyCode::Down => app.scroll_down(),
                    KeyCode::Char('k') | KeyCode::Up => app.scroll_up(),
                    _ => {}
                },
                app::Mode::Sending => {
                    if key.code == KeyCode::Esc {
                        app.mode = app::Mode::Normal;
                    }
                }
            }
        }
    }
}
