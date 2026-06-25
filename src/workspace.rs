//! Workspace switcher — multiple independent pane layouts.
use crate::layout::LayoutManager;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

const MAX_WORKSPACES: usize = 9;

pub struct WorkspaceManager {
    workspaces: Vec<LayoutManager>,
    pub current: usize,
}

impl WorkspaceManager {
    pub fn new() -> Self {
        Self {
            workspaces: vec![LayoutManager::new()],
            current: 0,
        }
    }

    pub fn current_layout(&mut self) -> &mut LayoutManager {
        &mut self.workspaces[self.current]
    }

    /// Switch to workspace by index (0-based). Creates it if missing.
    pub fn switch_to(&mut self, index: usize) {
        let index = index.min(MAX_WORKSPACES - 1);
        while self.workspaces.len() <= index {
            self.workspaces.push(LayoutManager::new());
        }
        self.current = index;
    }

    pub fn next(&mut self) {
        self.switch_to((self.current + 1) % self.workspaces.len().max(2).min(MAX_WORKSPACES));
    }

    pub fn prev(&mut self) {
        if self.current == 0 {
            self.switch_to(self.workspaces.len() - 1);
        } else {
            self.switch_to(self.current - 1);
        }
    }

    pub fn count(&self) -> usize {
        self.workspaces.len()
    }

    /// Render the workspace bar (e.g. [1] 2  3).
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let spans: Vec<Span> = (0..self.workspaces.len())
            .map(|i| {
                if i == self.current {
                    Span::styled(
                        format!(" [{}] ", i + 1),
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::styled(
                        format!("  {}  ", i + 1),
                        Style::default().fg(Color::DarkGray),
                    )
                }
            })
            .collect();

        let p = Paragraph::new(Line::from(spans))
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default().bg(Color::Reset));
        frame.render_widget(p, area);
    }
}