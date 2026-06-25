//! Simple in-TUI notification daemon.
//! Receives messages over mpsc and renders a toast overlay.
use std::collections::VecDeque;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

const MAX_VISIBLE: usize = 3;
const TOAST_TTL: Duration = Duration::from_secs(4);

struct Toast {
    message: String,
    born: Instant,
}

pub struct NotificationDaemon {
    rx: mpsc::Receiver<String>,
    pub tx: mpsc::Sender<String>,
    toasts: VecDeque<Toast>,
}

impl NotificationDaemon {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            rx,
            tx,
            toasts: VecDeque::new(),
        }
    }

    /// Clone the sender so other modules can push notifications.
    pub fn sender(&self) -> mpsc::Sender<String> {
        self.tx.clone()
    }

    /// Drain incoming messages and expire old toasts.
    pub fn tick(&mut self) {
        // Receive new
        while let Ok(msg) = self.rx.try_recv() {
            self.toasts.push_back(Toast {
                message: msg,
                born: Instant::now(),
            });
        }
        // Expire old
        self.toasts.retain(|t| t.born.elapsed() < TOAST_TTL);
        // Cap
        while self.toasts.len() > MAX_VISIBLE {
            self.toasts.pop_front();
        }
    }

    /// Render toast stack in the top-right corner.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if self.toasts.is_empty() {
            return;
        }
        let width: u16 = 36;
        let height: u16 = 3;
        let mut y = area.y + 1;

        for toast in self.toasts.iter().rev().take(MAX_VISIBLE) {
            if y + height > area.y + area.height {
                break;
            }
            let x = area.x + area.width.saturating_sub(width + 1);
            let rect = Rect::new(x, y, width, height);

            let elapsed = toast.born.elapsed().as_secs_f32();
            let ttl = TOAST_TTL.as_secs_f32();
            let fading = elapsed > ttl * 0.7;

            let color = if fading { Color::DarkGray } else { Color::Cyan };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(color))
                .title(Span::styled(
                    " TDE ",
                    Style::default()
                        .fg(color)
                        .add_modifier(Modifier::BOLD),
                ));

            let msg = if toast.message.len() > (width as usize - 4) {
                format!("{}…", &toast.message[..width as usize - 5])
            } else {
                toast.message.clone()
            };

            let p = Paragraph::new(Line::from(Span::raw(msg))).block(block);
            frame.render_widget(Clear, rect);
            frame.render_widget(p, rect);
            y += height + 1;
        }
    }
}