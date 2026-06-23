mod config;
mod launcher;
mod layout;
mod statusbar;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use layout::{LayoutManager, SplitDir};
use launcher::Launcher;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};
use statusbar::Statusbar;
use std::{
    io,
    time::{Duration, Instant},
};

fn main() -> Result<()> {
    let cfg = config::Config::load()?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, cfg);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(e) = res {
        eprintln!("Error: {e}");
    }
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, cfg: config::Config) -> Result<()> {
    let mut layout_mgr = LayoutManager::new();
    let mut launcher = Launcher::new();
    let mut statusbar = Statusbar::new(cfg.statusbar.clone());

    let tick = Duration::from_millis(500);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| {
            let size = frame.area();

            // Split area: statusbar (1 row) + main content
            let root = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(size);

            let content_area = root[0];
            let bar_area = root[1];

            // Render panes
            let rects = layout_mgr.compute_rects(content_area);
            for (i, pane) in layout_mgr.panes.iter().enumerate() {
                if let Some(&rect) = rects.get(i) {
                    let is_focused = i == layout_mgr.focused;
                    let border_style = if is_focused {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .border_type(if is_focused { BorderType::Thick } else { BorderType::Plain })
                        .border_style(border_style)
                        .title(format!(" {} ", pane.title))
                        .title_style(Style::default().add_modifier(Modifier::BOLD));

                    let help = Paragraph::new(
                        "  Press SPACE to open launcher\n  Tab / BackTab — switch panes\n  H / V — split pane\n  X — close pane\n  Q / Ctrl-C — quit"
                    )
                    .block(block)
                    .style(Style::default().fg(Color::DarkGray));
                    frame.render_widget(help, rect);
                }
            }

            // Statusbar
            statusbar.render(frame, bar_area);

            // Launcher overlay
            if launcher.visible {
                launcher.render(frame, size);
            }
        })?;

        // Tick: refresh sysinfo
        if last_tick.elapsed() >= tick {
            statusbar.refresh();
            last_tick = Instant::now();
        }

        // Events
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                // Ctrl-C always quits
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c') {
                    return Ok(());
                }

                if launcher.visible {
                    match key.code {
                        KeyCode::Esc           => launcher.hide(),
                        KeyCode::Down          => launcher.select_next(),
                        KeyCode::Up            => launcher.select_prev(),
                        KeyCode::Backspace     => launcher.pop_char(),
                        KeyCode::Enter         => {
                            if let Some(app) = launcher.selected_app() {
                                // In a real TDE, we'd spawn this inside the focused pane.
                                // For now just log it.
                                let _ = app;
                            }
                            launcher.hide();
                        }
                        KeyCode::Char(c)       => launcher.push_char(c),
                        _                      => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q')     => return Ok(()),
                        KeyCode::Char(' ')     => launcher.show(),
                        KeyCode::Tab           => layout_mgr.focus_next(),
                        KeyCode::BackTab       => layout_mgr.focus_prev(),
                        KeyCode::Char('h')     => layout_mgr.split_pane(SplitDir::Horizontal),
                        KeyCode::Char('v')     => layout_mgr.split_pane(SplitDir::Vertical),
                        KeyCode::Char('x')     => layout_mgr.close_focused(),
                        _                      => {}
                    }
                }
            }
        }
    }
}