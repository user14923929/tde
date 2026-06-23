/// Manages the tiling layout of panes on screen.
use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Debug, Clone, PartialEq)]
pub enum SplitDir {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub struct Pane {
    #[allow(dead_code)]
    pub id: usize,
    pub title: String,
}

impl Pane {
    pub fn new(id: usize, title: impl Into<String>) -> Self {
        Self { id, title: title.into() }
    }
}

#[derive(Debug)]
pub struct LayoutManager {
    pub panes: Vec<Pane>,
    pub focused: usize,
    pub split: SplitDir,
    next_id: usize,
}

impl LayoutManager {
    pub fn new() -> Self {
        Self {
            panes: vec![Pane::new(0, "Shell")],
            focused: 0,
            split: SplitDir::Horizontal,
            next_id: 1,
        }
    }

    pub fn split_pane(&mut self, dir: SplitDir) {
        self.split = dir;
        let id = self.next_id;
        self.next_id += 1;
        self.panes.push(Pane::new(id, "Shell"));
        self.focused = self.panes.len() - 1;
    }

    pub fn close_focused(&mut self) {
        if self.panes.len() <= 1 { return; }
        self.panes.remove(self.focused);
        if self.focused >= self.panes.len() {
            self.focused = self.panes.len() - 1;
        }
    }

    pub fn focus_next(&mut self) {
        if self.panes.is_empty() { return; }
        self.focused = (self.focused + 1) % self.panes.len();
    }

    pub fn focus_prev(&mut self) {
        if self.panes.is_empty() { return; }
        self.focused = (self.focused + self.panes.len() - 1) % self.panes.len();
    }

    /// Returns a Rect per pane, tiled inside `area`.
    pub fn compute_rects(&self, area: Rect) -> Vec<Rect> {
        if self.panes.is_empty() { return vec![]; }
        let n = self.panes.len() as u16;
        let dir = match self.split {
            SplitDir::Horizontal => Direction::Horizontal,
            SplitDir::Vertical   => Direction::Vertical,
        };
        let constraints: Vec<Constraint> = (0..n)
            .map(|_| Constraint::Ratio(1, n as u32))
            .collect();
        Layout::default()
            .direction(dir)
            .constraints(constraints)
            .split(area)
            .iter()
            .cloned()
            .collect()
    }
}