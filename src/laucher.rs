use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

static APPS: &[(&str, &str)] = &[
    ("htop",     "Interactive process viewer"),
    ("nvim",     "Neovim text editor"),
    ("lf",       "Terminal file manager"),
    ("lazygit",  "Git TUI"),
    ("btop",     "Resource monitor"),
    ("ncdu",     "Disk usage analyzer"),
    ("cava",     "Audio visualizer"),
    ("nnn",      "File manager"),
];

pub struct Launcher {
    pub visible: bool,
    pub query: String,
    pub list_state: ListState,
    pub filtered: Vec<(&'static str, &'static str)>,
}

impl Launcher {
    pub fn new() -> Self {
        let mut s = Self {
            visible: false,
            query: String::new(),
            list_state: ListState::default(),
            filtered: APPS.to_vec(),
        };
        s.list_state.select(Some(0));
        s
    }

    pub fn show(&mut self) { self.visible = true; }
    pub fn hide(&mut self) {
        self.visible = false;
        self.query.clear();
        self.filter();
    }

    pub fn push_char(&mut self, c: char) {
        self.query.push(c);
        self.filter();
    }

    pub fn pop_char(&mut self) {
        self.query.pop();
        self.filter();
    }

    pub fn select_next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => (i + 1).min(self.filtered.len().saturating_sub(1)),
            None    => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn select_prev(&mut self) {
        let i = self.list_state.selected().unwrap_or(0).saturating_sub(1);
        self.list_state.select(Some(i));
    }

    pub fn selected_app(&self) -> Option<&str> {
        let i = self.list_state.selected()?;
        self.filtered.get(i).map(|(name, _)| *name)
    }

    fn filter(&mut self) {
        let q = self.query.to_lowercase();
        self.filtered = APPS.iter()
            .filter(|(name, desc)| {
                name.contains(&*q) || desc.to_lowercase().contains(&*q)
            })
            .cloned()
            .collect();
        let max = self.filtered.len().saturating_sub(1);
        let cur = self.list_state.selected().unwrap_or(0).min(max);
        self.list_state.select(if self.filtered.is_empty() { None } else { Some(cur) });
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Center a popup
        let popup = centered_rect(50, 60, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(popup);

        // Search input
        let input = Paragraph::new(format!(" > {}", self.query))
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Launcher ")
                .style(Style::default().fg(Color::Cyan)));
        frame.render_widget(input, chunks[0]);

        // App list
        let items: Vec<ListItem> = self.filtered.iter().map(|(name, desc)| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<12}", name), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled(*desc, Style::default().fg(Color::DarkGray)),
            ]))
        }).collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                .border_type(BorderType::Rounded))
            .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black));
        frame.render_stateful_widget(list, chunks[1], &mut self.list_state);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(layout[1])[1]
}