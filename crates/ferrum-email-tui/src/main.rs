mod app;
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

    let mut app = App::new().await?;
    let result = run_app(&mut terminal, &mut app).await;

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
