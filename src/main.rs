mod config;
mod launcher;
mod layout;
mod notifications;
mod plugin;
mod pty;
mod statusbar;
mod workspace;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use launcher::Launcher;
use notifications::NotificationDaemon;
use plugin::PluginManager;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};
use statusbar::Statusbar;
use workspace::WorkspaceManager;

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
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = res {
        eprintln!("Error: {e}");
    }
    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    cfg: config::Config,
) -> Result<()> {
    // --- subsystems ---
    let mut workspaces = WorkspaceManager::new();
    let mut launcher = Launcher::new();
    let mut statusbar = Statusbar::new(cfg.statusbar.clone());
    let mut notif = NotificationDaemon::new();

    // Lua plugin system
    let plugins = PluginManager::new()?;
    plugins.register_api(notif.sender())?;
    plugins.load_all()?;
    plugins.call_hook("on_startup")?;

    notif.tx.send("TDE v0.1.1 started".into()).ok();

    let tick = Duration::from_millis(500);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| {
            let size = frame.area();

            // Layout: workspace bar (1) | content (min) | statusbar (1)
            let root = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Min(1),
                    Constraint::Length(1),
                ])
                .split(size);

            let ws_area = root[0];
            let content_area = root[1];
            let bar_area = root[2];

            // Workspace bar
            workspaces.render(frame, ws_area);

            // Panes
            let layout_mgr = workspaces.current_layout();
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
                        .border_type(if is_focused {
                            BorderType::Thick
                        } else {
                            BorderType::Plain
                        })
                        .border_style(border_style)
                        .title(format!(" {} ", pane.title))
                        .title_style(Style::default().add_modifier(Modifier::BOLD));

                    let help = Paragraph::new(
                        "  SPACE — launcher    Tab/BackTab — focus\n  H/V — split          X — close pane\n  1-9 — workspace      Q/Ctrl-C — quit",
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

            // Notification toasts
            notif.render(frame, size);
        })?;

        // Tick
        if last_tick.elapsed() >= tick {
            statusbar.refresh();
            notif.tick();
            last_tick = Instant::now();
        }

        // Events
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                // Ctrl-C: always quit
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c') {
                    return Ok(());
                }

                if launcher.visible {
                    match key.code {
                        KeyCode::Esc => launcher.hide(),
                        KeyCode::Down => launcher.select_next(),
                        KeyCode::Up => launcher.select_prev(),
                        KeyCode::Backspace => launcher.pop_char(),
                        KeyCode::Enter => {
                            if let Some(app) = launcher.selected_app().map(str::to_owned) {
                                launcher.hide();
                                disable_raw_mode()?;
                                execute!(
                                    terminal.backend_mut(),
                                    LeaveAlternateScreen,
                                    DisableMouseCapture
                                )?;
                                terminal.show_cursor()?;

                                std::process::Command::new(&app).status().ok();

                                enable_raw_mode()?;
                                execute!(
                                    terminal.backend_mut(),
                                    EnterAlternateScreen,
                                    EnableMouseCapture
                                )?;
                                terminal.clear()?;
                                notif.tx.send(format!("{app} exited")).ok();
                            } else {
                                launcher.hide();
                            }
                        }
                        KeyCode::Char(c) => launcher.push_char(c),
                        _ => {}
                    }
                } else {
                    let layout_mgr = workspaces.current_layout();
                    match key.code {
                        // Quit
                        KeyCode::Char('q') => return Ok(()),
                        // Launcher
                        KeyCode::Char(' ') => launcher.show(),
                        // Pane focus
                        KeyCode::Tab => layout_mgr.focus_next(),
                        KeyCode::BackTab => layout_mgr.focus_prev(),
                        // Pane splits
                        KeyCode::Char('h') => {
                            layout_mgr.split_pane(layout::SplitDir::Horizontal)
                        }
                        KeyCode::Char('v') => layout_mgr.split_pane(layout::SplitDir::Vertical),
                        KeyCode::Char('x') => layout_mgr.close_focused(),
                        // Workspace switching: 1-9
                        KeyCode::Char(c @ '1'..='9') => {
                            let idx = c as usize - '1' as usize;
                            workspaces.switch_to(idx);
                            notif
                                .tx
                                .send(format!("Workspace {}", idx + 1))
                                .ok();
                        }
                        // Workspace prev/next
                        KeyCode::Char('[') => workspaces.prev(),
                        KeyCode::Char(']') => workspaces.next(),
                        _ => {}
                    }
                }
            }
        }
    }
}