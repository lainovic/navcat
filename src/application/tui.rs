use std::io;
use std::process::Child;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{List, ListItem, Paragraph},
    Terminal,
};

use crate::domain::filter::LogFilter;
use crate::domain::filter_config::FilterState;

const MAX_BUFFER: usize = 50_000;
const TRIM_SIZE: usize = 10_000;

pub struct AppState {
    raw_buffer: Vec<String>,
    pub filter_state: FilterState,
    filter: LogFilter,
    scroll_offset: usize,
    follow: bool,
}

impl AppState {
    pub fn new(filter_state: FilterState) -> Self {
        let filter = LogFilter::new(filter_state.to_filter_config());
        Self {
            raw_buffer: Vec::new(),
            filter_state,
            filter,
            scroll_offset: 0,
            follow: true,
        }
    }

    fn rebuild_filter(&mut self) {
        self.filter = LogFilter::new(self.filter_state.to_filter_config());
    }

    pub fn toggle_guidance(&mut self) {
        self.filter_state.guidance = !self.filter_state.guidance;
        self.rebuild_filter();
    }

    pub fn toggle_routing(&mut self) {
        self.filter_state.routing = !self.filter_state.routing;
        self.rebuild_filter();
    }

    pub fn toggle_mapmatching(&mut self) {
        self.filter_state.mapmatching = !self.filter_state.mapmatching;
        self.rebuild_filter();
    }

    pub fn push_line(&mut self, line: String) {
        self.raw_buffer.push(line);
        if self.raw_buffer.len() > MAX_BUFFER {
            self.raw_buffer.drain(..TRIM_SIZE);
            self.scroll_offset = self.scroll_offset.saturating_sub(TRIM_SIZE);
        }
    }

    pub fn filtered_lines(&self) -> Vec<String> {
        self.raw_buffer
            .iter()
            .filter_map(|line| self.filter.matches(line))
            .collect()
    }

    pub fn scroll_up(&mut self) {
        self.follow = false;
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.follow = false;
        self.scroll_offset += 1; // clamped in render
    }

    pub fn resume_follow(&mut self) {
        self.follow = true;
    }
}

pub fn run_tui(
    mut child: Child,
    receiver: Receiver<String>,
    filter_state: FilterState,
) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = AppState::new(filter_state);
    let result = run_loop(&mut terminal, &mut app, &receiver);

    // Always restore terminal, even on error
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();
    let _ = child.kill();

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut AppState,
    receiver: &Receiver<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut dirty = true;

    loop {
        // Drain new lines from the adb thread
        let mut received = false;
        while let Ok(line) = receiver.try_recv() {
            app.push_line(line);
            received = true;
        }
        if received {
            dirty = true;
        }

        if dirty {
            let filtered = app.filtered_lines();
            terminal.draw(|frame| render(app, &filtered, frame))?;
            dirty = false;
        }

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('g') => {
                        app.toggle_guidance();
                        dirty = true;
                    }
                    KeyCode::Char('r') => {
                        app.toggle_routing();
                        dirty = true;
                    }
                    KeyCode::Char('m') => {
                        app.toggle_mapmatching();
                        dirty = true;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.scroll_up();
                        dirty = true;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.scroll_down();
                        dirty = true;
                    }
                    KeyCode::Char('f') | KeyCode::End => {
                        app.resume_follow();
                        dirty = true;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn render(app: &AppState, filtered: &[String], frame: &mut ratatui::Frame) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let log_area = chunks[0];
    let status_area = chunks[1];
    let height = log_area.height as usize;

    let scroll_offset = if app.follow {
        filtered.len().saturating_sub(height)
    } else {
        app.scroll_offset
            .min(filtered.len().saturating_sub(1).max(0))
    };

    if filtered.is_empty() {
        frame.render_widget(splash(), log_area);
    } else {
        let items: Vec<ListItem> = filtered
            .iter()
            .skip(scroll_offset)
            .take(height)
            .map(|line| ListItem::new(ansi_to_line(line)))
            .collect();
        frame.render_widget(List::new(items), log_area);
    }

    let g = if app.filter_state.guidance { "g:on " } else { "g:off" };
    let r = if app.filter_state.routing { "r:on " } else { "r:off" };
    let m = if app.filter_state.mapmatching { "m:on " } else { "m:off" };
    let mode = if app.follow { "FOLLOW" } else { "LOCKED" };
    let status = format!(
        " [{} {} {}] │ {} lines │ {}  │  g/r/m:toggle  ↑↓ jk:scroll  f:follow  q:quit",
        g,
        r,
        m,
        filtered.len(),
        mode
    );

    frame.render_widget(
        Paragraph::new(status)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White)),
        status_area,
    );
}

fn splash() -> Paragraph<'static> {
    let red = Style::default().fg(Color::Red);
    let bold_white = Style::default().fg(Color::White).add_modifier(Modifier::BOLD);
    let dim = Style::default().fg(Color::DarkGray);

    let key = Style::default().fg(Color::White).add_modifier(Modifier::BOLD);

    let text = Text::from(vec![
        Line::from(""),
        Line::from(vec![Span::styled(" /\\_/\\  ", red), Span::styled("navcat", bold_white)]),
        Line::from(vec![Span::styled("( o.o )  ", red), Span::styled("nav log inspector", dim)]),
        Line::from(vec![Span::styled(" > ^ <", red)]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  g", key), Span::styled("  guidance    ", dim),
            Span::styled("r", key),   Span::styled("  routing    ", dim),
            Span::styled("m", key),   Span::styled("  map-matching", dim),
        ]),
        Line::from(vec![
            Span::styled("  ↑↓", key), Span::styled(" scroll      ", dim),
            Span::styled("f", key),    Span::styled("  follow     ", dim),
            Span::styled("q", key),    Span::styled("  quit", dim),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("  waiting for logs...", dim)]),
    ]);

    Paragraph::new(text).alignment(Alignment::Left)
}

/// Converts a string containing ANSI escape codes into a ratatui `Line` with
/// styled `Span`s. Only handles the specific escape codes this app generates.
fn ansi_to_line(s: &str) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut current_style = Style::default();
    let mut current_text = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next(); // consume '['
            if !current_text.is_empty() {
                spans.push(Span::styled(
                    std::mem::take(&mut current_text),
                    current_style,
                ));
            }
            let mut code = String::new();
            for ch in chars.by_ref() {
                code.push(ch);
                if ch.is_ascii_alphabetic() {
                    break;
                }
            }
            current_style = match code.as_str() {
                "0m" => Style::default(),
                "31m" => Style::default().fg(Color::Red),
                "32m" => Style::default().fg(Color::Green),
                "33m" => Style::default().fg(Color::Yellow),
                "34m" => Style::default().fg(Color::Blue),
                "35m" => Style::default().fg(Color::Magenta),
                "36m" => Style::default().fg(Color::Cyan),
                "1;31m" => Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
                "1;32m" => Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
                "43m" => Style::default().bg(Color::Yellow),
                _ => Style::default(),
            };
        } else {
            current_text.push(c);
        }
    }

    if !current_text.is_empty() {
        spans.push(Span::styled(current_text, current_style));
    }

    Line::from(spans)
}
