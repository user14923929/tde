use crate::config::StatusbarConfig;
use chrono::Local;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};
use sysinfo::System;

pub struct Statusbar {
    pub cfg: StatusbarConfig,
    sys: System,
}

impl Statusbar {
    pub fn new(cfg: StatusbarConfig) -> Self {
        Self { cfg, sys: System::new_all() }
    }

    pub fn refresh(&mut self) {
        self.sys.refresh_all();
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let mut segments: Vec<Span> = vec![];

        // Left: branding
        segments.push(Span::styled(" TDE ", Style::default().fg(Color::Cyan)));
        segments.push(Span::raw("│ "));

        // CPU
        if self.cfg.show_cpu {
            let cpu = self.sys.global_cpu_usage();
            segments.push(Span::raw(format!("CPU {:.0}%  ", cpu)));
        }

        // RAM
        if self.cfg.show_mem {
            let used = self.sys.used_memory() / 1024 / 1024;
            let total = self.sys.total_memory() / 1024 / 1024;
            segments.push(Span::raw(format!("MEM {}/{}MB  ", used, total)));
        }

        // Filler
        segments.push(Span::raw("─".repeat(4)));

        // Clock (right-aligned via a separate paragraph is complex,
        // so we just append at the end)
        if self.cfg.show_clock {
            let now = Local::now().format("%H:%M:%S").to_string();
            segments.push(Span::styled(format!(" {} ", now), Style::default().fg(Color::Yellow)));
        }

        let line = Line::from(segments);
        let p = Paragraph::new(line)
            .block(Block::default())
            .style(Style::default().fg(Color::White).bg(Color::DarkGray));
        frame.render_widget(p, area);
    }
}