use std::io;
use std::process::Child;
use std::sync::mpsc::Receiver;

use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
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
const FLASH_MS: u64 = 350;

pub struct AppState {
    raw_buffer: Vec<String>,
    filtered_cache: Vec<String>,
    pub filter_state: FilterState,
    filter: LogFilter,
    scroll_offset: usize,
    follow: bool,
    flash: Option<(Instant, char)>,
    pub visible_height: usize,
    show_hint: bool,
    search_mode: bool,
    search_query: String,
    quit_pending: Option<Instant>,
    save_notice: Option<(Instant, String)>,
    pub adb_connected: bool,
}

impl AppState {
    pub fn new(filter_state: FilterState) -> Self {
        let filter = LogFilter::new(filter_state.to_filter_config());
        Self {
            raw_buffer: Vec::new(),
            filtered_cache: Vec::new(),
            filter_state,
            filter,
            scroll_offset: 0,
            follow: true,
            flash: None,
            visible_height: 24,
            show_hint: false,
            search_mode: false,
            search_query: String::new(),
            quit_pending: None,
            save_notice: None,
            adb_connected: true,
        }
    }

    fn rebuild_filter(&mut self) {
        self.filter = LogFilter::new(self.filter_state.to_filter_config());
        self.filtered_cache = self.raw_buffer
            .iter()
            .filter_map(|line| self.filter.matches(line))
            .collect();
    }

    fn set_flash(&mut self, key: char) {
        self.flash = Some((Instant::now() + Duration::from_millis(FLASH_MS), key));
    }

    fn is_flashing(&self, key: char) -> bool {
        self.flash
            .map_or(false, |(until, k)| k == key && Instant::now() < until)
    }

    pub fn toggle_navigation(&mut self) {
        self.filter_state.navigation = !self.filter_state.navigation;
        self.rebuild_filter();
        self.set_flash('n');
    }

    pub fn toggle_guidance(&mut self) {
        self.filter_state.guidance = !self.filter_state.guidance;
        self.rebuild_filter();
        self.set_flash('g');
    }

    pub fn toggle_routing(&mut self) {
        self.filter_state.routing = !self.filter_state.routing;
        self.rebuild_filter();
        self.set_flash('r');
    }

    pub fn toggle_mapmatching(&mut self) {
        self.filter_state.mapmatching = !self.filter_state.mapmatching;
        self.rebuild_filter();
        self.set_flash('m');
    }

    pub fn toggle_hint(&mut self) {
        self.show_hint = !self.show_hint;
    }

    pub fn reset_filters(&mut self) {
        self.filter_state.navigation = true;
        self.filter_state.guidance = true;
        self.filter_state.routing = true;
        self.filter_state.mapmatching = true;
        self.search_query.clear();
        self.search_mode = false;
        self.rebuild_filter();
    }

    pub fn clear_filters(&mut self) {
        self.filter_state.navigation = false;
        self.filter_state.guidance = false;
        self.filter_state.routing = false;
        self.filter_state.mapmatching = false;
        self.rebuild_filter();
    }

    pub fn dump_to_file(&self) -> Result<String, std::io::Error> {
        use std::io::Write;
        let filename = {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            format!("navcat_{}.txt", now.as_secs())
        };
        let mut file = std::fs::File::create(&filename)?;
        for line in &self.raw_buffer {
            writeln!(file, "{}", line)?;
        }
        Ok(filename)
    }

    pub fn enter_search(&mut self) {
        self.search_mode = true;
    }

    pub fn exit_search(&mut self, clear: bool) {
        self.search_mode = false;
        if clear {
            self.search_query.clear();
        }
    }

    pub fn search_push(&mut self, c: char) {
        self.search_query.push(c);
        self.follow = true; // jump to latest match as query narrows
    }

    pub fn search_pop(&mut self) {
        self.search_query.pop();
        self.follow = true;
    }

    pub fn has_search(&self) -> bool {
        !self.search_query.is_empty()
    }

    pub fn push_line(&mut self, line: String) {
        if let Some(filtered) = self.filter.matches(&line) {
            self.filtered_cache.push(filtered);
        }
        self.raw_buffer.push(line);
        if self.raw_buffer.len() > MAX_BUFFER {
            self.raw_buffer.drain(..TRIM_SIZE);
            self.scroll_offset = self.scroll_offset.saturating_sub(TRIM_SIZE);
            // Rebuild cache after trim since raw/filtered are now out of sync
            self.filtered_cache = self.raw_buffer
                .iter()
                .filter_map(|l| self.filter.matches(l))
                .collect();
        }
    }

    pub fn raw_count(&self) -> usize {
        self.raw_buffer.len()
    }

    pub fn filtered_lines(&self) -> &[String] {
        &self.filtered_cache
    }

    fn leave_follow(&mut self) {
        if self.follow {
            // Sync to the actual bottom position before disabling follow,
            // otherwise scroll_offset of 0 would jump the view to the top.
            self.scroll_offset = self.filtered_cache.len().saturating_sub(self.visible_height);
            self.follow = false;
        }
    }

    pub fn scroll_up(&mut self) {
        self.leave_follow();
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.leave_follow();
        self.scroll_offset += 1; // clamped in render
    }

    pub fn scroll_page_up(&mut self) {
        self.leave_follow();
        let step = (self.visible_height / 2).max(1);
        self.scroll_offset = self.scroll_offset.saturating_sub(step);
    }

    pub fn scroll_page_down(&mut self) {
        self.leave_follow();
        let step = (self.visible_height / 2).max(1);
        self.scroll_offset += step; // clamped in render
    }

    pub fn resume_follow(&mut self) {
        self.follow = true;
    }

    pub fn clear_buffer(&mut self) {
        self.raw_buffer.clear();
        self.filtered_cache.clear();
        self.scroll_offset = 0;
        self.follow = true;
    }
}

pub fn run_tui(
    mut child: Option<Child>,
    receiver: Option<Receiver<String>>,
    filter_state: FilterState,
    preloaded: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = AppState::new(filter_state);

    if !preloaded.is_empty() {
        for line in preloaded {
            app.push_line(line);
        }
        // follow stays true — start at bottom (most recent events) for file mode
    }

    let result = run_loop(&mut terminal, &mut app, &receiver, &mut child);

    // Always restore terminal, even on error
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();
    if let Some(ref mut c) = child {
        let _ = c.kill();
    }

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut AppState,
    receiver: &Option<Receiver<String>>,
    child: &mut Option<Child>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut dirty = true;

    loop {
        // Drain new lines from the adb thread (live mode only)
        if let Some(rx) = receiver {
            let mut received = false;
            while let Ok(line) = rx.try_recv() {
                app.push_line(line);
                received = true;
            }
            if received {
                app.adb_connected = true; // clear disconnect banner when lines resume
                dirty = true;
            }
        }

        // Detect adb subprocess exit
        if app.adb_connected {
            if let Some(c) = child {
                if matches!(c.try_wait(), Ok(Some(_))) {
                    app.adb_connected = false;
                    dirty = true;
                }
            }
        }

        // Expire toggle flash and trigger one final redraw when it ends
        if let Some((until, _)) = app.flash {
            if Instant::now() < until {
                dirty = true;
            } else {
                app.flash = None;
                dirty = true;
            }
        }

        // Expire save notice
        if let Some((deadline, _)) = app.save_notice {
            if Instant::now() >= deadline {
                app.save_notice = None;
                dirty = true;
            } else {
                dirty = true;
            }
        }

        // Expire quit confirmation window
        if let Some(deadline) = app.quit_pending {
            if Instant::now() >= deadline {
                app.quit_pending = None;
                dirty = true;
            } else {
                dirty = true; // keep redrawing so status bar stays live
            }
        }

        if dirty {
            // Update visible_height before render so page-scroll has correct size
            if let Ok(size) = terminal.size() {
                app.visible_height = (size.height as usize).saturating_sub(1);
            }
            let filtered = app.filtered_lines();
            terminal.draw(|frame| render(app, filtered, frame))?;
            dirty = false;
        }

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if app.search_mode {
                    match key {
                        KeyEvent { code: KeyCode::Esc, .. } => {
                            app.exit_search(true);
                            dirty = true;
                        }
                        KeyEvent { code: KeyCode::Enter, .. } => {
                            app.exit_search(false);
                            dirty = true;
                        }
                        KeyEvent { code: KeyCode::Backspace, .. } => {
                            app.search_pop();
                            dirty = true;
                        }
                        KeyEvent { code: KeyCode::Char(c), modifiers: KeyModifiers::NONE, .. }
                        | KeyEvent { code: KeyCode::Char(c), modifiers: KeyModifiers::SHIFT, .. } => {
                            app.search_push(c);
                            dirty = true;
                        }
                        // Pass scroll keys through so you can browse while typing
                        KeyEvent { code: KeyCode::PageUp, .. }
                        | KeyEvent { code: KeyCode::Char('u'), modifiers: KeyModifiers::CONTROL, .. } => {
                            app.scroll_page_up();
                            dirty = true;
                        }
                        KeyEvent { code: KeyCode::PageDown, .. }
                        | KeyEvent { code: KeyCode::Char('d'), modifiers: KeyModifiers::CONTROL, .. } => {
                            app.scroll_page_down();
                            dirty = true;
                        }
                        _ => {}
                    }
                } else {
                    match key {
                        KeyEvent { code: KeyCode::Char('q'), .. } => {
                            if app.quit_pending.map_or(false, |d| Instant::now() < d) {
                                break;
                            }
                            app.quit_pending = Some(Instant::now() + Duration::from_millis(1500));
                            dirty = true;
                        }
                        KeyEvent { code: KeyCode::Char('/'), .. } => {
                            app.enter_search();
                            dirty = true;
                        }
                        KeyEvent { code: KeyCode::Esc, .. } if app.has_search() => {
                            app.exit_search(true);
                            dirty = true;
                        }
                        KeyEvent { code: KeyCode::Char('0'), .. } => {
                            app.reset_filters();
                            dirty = true;
                        }
                        KeyEvent { code: KeyCode::Char('-'), .. } => {
                            app.clear_filters();
                            dirty = true;
                        }
                        KeyEvent { code: KeyCode::Char('n'), .. } => {
                            app.toggle_navigation();
                            dirty = true;
                        }
                        KeyEvent { code: KeyCode::Char('g'), .. } => {
                            app.toggle_guidance();
                            dirty = true;
                        }
                        KeyEvent { code: KeyCode::Char('r'), .. } => {
                            app.toggle_routing();
                            dirty = true;
                        }
                        KeyEvent { code: KeyCode::Char('m'), .. } => {
                            app.toggle_mapmatching();
                            dirty = true;
                        }
                        KeyEvent { code: KeyCode::Char('w'), .. } => {
                            let msg = match app.dump_to_file() {
                                Ok(filename) => format!("  saved to {}", filename),
                                Err(e) => format!("  save failed: {}", e),
                            };
                            app.save_notice = Some((Instant::now() + Duration::from_millis(3000), msg));
                            dirty = true;
                        }
                        KeyEvent { code: KeyCode::Char('?'), .. } => {
                            app.toggle_hint();
                            dirty = true;
                        }
                        KeyEvent { code: KeyCode::Up, .. } | KeyEvent { code: KeyCode::Char('k'), .. } => {
                            app.scroll_up();
                            dirty = true;
                        }
                        KeyEvent { code: KeyCode::Down, .. } | KeyEvent { code: KeyCode::Char('j'), .. } => {
                            app.scroll_down();
                            dirty = true;
                        }
                        KeyEvent { code: KeyCode::PageUp, .. }
                        | KeyEvent { code: KeyCode::Char('u'), modifiers: KeyModifiers::CONTROL, .. } => {
                            app.scroll_page_up();
                            dirty = true;
                        }
                        KeyEvent { code: KeyCode::PageDown, .. }
                        | KeyEvent { code: KeyCode::Char('d'), modifiers: KeyModifiers::CONTROL, .. } => {
                            app.scroll_page_down();
                            dirty = true;
                        }
                        KeyEvent { code: KeyCode::Char('f'), .. } | KeyEvent { code: KeyCode::End, .. } => {
                            app.resume_follow();
                            dirty = true;
                        }
                        KeyEvent { code: KeyCode::Char('l'), modifiers: KeyModifiers::CONTROL, .. } => {
                            app.clear_buffer();
                            dirty = true;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

fn render(app: &AppState, filtered: &[String], frame: &mut ratatui::Frame) {
    let area = frame.area();

    let constraints: Vec<Constraint> = if app.search_mode {
        vec![Constraint::Min(1), Constraint::Length(1), Constraint::Length(1)]
    } else {
        vec![Constraint::Min(1), Constraint::Length(1)]
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let log_area = chunks[0];
    let (search_area, status_area) = if app.search_mode {
        (Some(chunks[1]), chunks[2])
    } else {
        (None, chunks[1])
    };
    let height = log_area.height as usize;

    // Apply search on top of category filter
    let search_q = app.search_query.to_lowercase();
    let display: Vec<&String> = if search_q.is_empty() {
        filtered.iter().collect()
    } else {
        filtered
            .iter()
            .filter(|l| l.to_lowercase().contains(&search_q))
            .collect()
    };

    let scroll_offset = if app.follow {
        display.len().saturating_sub(height)
    } else {
        app.scroll_offset
            .min(display.len().saturating_sub(1).max(0))
    };

    if display.is_empty() && app.raw_count() == 0 {
        frame.render_widget(splash(), log_area);
    } else if display.is_empty() {
        let dim = Style::default().fg(Color::DarkGray);
        let msg = Paragraph::new(Line::from(vec![
            Span::styled("  no logs match current filters", dim),
        ]));
        frame.render_widget(msg, log_area);
    } else {
        let items: Vec<ListItem> = display
            .iter()
            .skip(scroll_offset)
            .take(height)
            .map(|line| ListItem::new(ansi_to_line(line)))
            .collect();
        frame.render_widget(List::new(items), log_area);
    }

    // Search bar
    if let Some(area) = search_area {
        let bar_style = Style::default().bg(Color::DarkGray).fg(Color::White);
        let cursor_style = Style::default().bg(Color::White).fg(Color::DarkGray);
        let search_line = Line::from(vec![
            Span::styled(" / ", bar_style),
            Span::styled(app.search_query.clone(), bar_style),
            Span::styled("█", cursor_style),
            Span::styled("  esc:clear  enter:lock", Style::default().bg(Color::DarkGray).fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        ]);
        frame.render_widget(Paragraph::new(search_line), area);
    }

    let base_style = Style::default().bg(Color::DarkGray).fg(Color::White);
    let flash_style = Style::default().bg(Color::White).fg(Color::DarkGray).add_modifier(Modifier::BOLD);

    // Color identifies the category; brightness indicates on/off state
    let toggle_style = |on: bool, key: char| -> Style {
        if app.is_flashing(key) {
            return flash_style;
        }
        let style = match key {
            'n' => Style::default().bg(Color::DarkGray).fg(Color::Blue),
            'g' => Style::default().bg(Color::DarkGray).fg(Color::Magenta),
            'r' => Style::default().bg(Color::DarkGray).fg(Color::Red).add_modifier(Modifier::BOLD),
            'm' => Style::default().bg(Color::DarkGray).fg(Color::Yellow),
            _   => Style::default().bg(Color::DarkGray).fg(Color::White),
        };
        if on { style } else { style.add_modifier(Modifier::DIM) }
    };

    let mode = if app.follow { "FOLLOW" } else { "PAUSED" };

    let pos = if app.follow {
        String::new()
    } else {
        let max_scroll = display.len().saturating_sub(height);
        if max_scroll == 0 || scroll_offset == 0 {
            " [top]".to_string()
        } else if scroll_offset >= max_scroll {
            " [bot]".to_string()
        } else {
            format!(" [{:2}%]", scroll_offset * 100 / max_scroll)
        }
    };

    let search_indicator = if !app.search_mode && app.has_search() {
        format!("  / \"{}\"", app.search_query)
    } else {
        String::new()
    };

    let quit_confirming = app.quit_pending.map_or(false, |d| Instant::now() < d);
    let save_msg = app.save_notice.as_ref()
        .filter(|(d, _)| Instant::now() < *d)
        .map(|(_, msg)| msg.as_str());
    let hint = if !app.adb_connected {
        "  adb disconnected — reconnecting..."
    } else if let Some(msg) = save_msg {
        msg
    } else if quit_confirming {
        "  press q again to quit"
    } else if app.show_hint {
        "  n/g/r/m:toggle  0:all on  -:all off  w:save  /:search  ↑↓ jk:scroll  PgUp/Dn ^u/d:page  f:follow  ^l:clear  q:quit  ?:hide"
    } else {
        "  ?"
    };

    let status_line = Line::from(vec![
        Span::styled(" [", base_style),
        Span::styled(
            if app.filter_state.navigation { "n:on " } else { "n:off" },
            toggle_style(app.filter_state.navigation, 'n'),
        ),
        Span::styled(" ", base_style),
        Span::styled(
            if app.filter_state.guidance { "g:on " } else { "g:off" },
            toggle_style(app.filter_state.guidance, 'g'),
        ),
        Span::styled(" ", base_style),
        Span::styled(
            if app.filter_state.routing { "r:on " } else { "r:off" },
            toggle_style(app.filter_state.routing, 'r'),
        ),
        Span::styled(" ", base_style),
        Span::styled(
            if app.filter_state.mapmatching { "m:on " } else { "m:off" },
            toggle_style(app.filter_state.mapmatching, 'm'),
        ),
        Span::styled(
            format!(
                "] │ {} / {} │ {}{}{}",
                display.len(),
                app.raw_count(),
                mode,
                pos,
                search_indicator,
            ),
            base_style,
        ),
        Span::styled(
            hint,
            if !app.adb_connected {
                Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)
            } else if quit_confirming {
                Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)
            } else if save_msg.is_some() {
                Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)
            } else {
                base_style
            },
        ),
    ]);

    frame.render_widget(Paragraph::new(status_line), status_area);
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
            Span::styled("  n", key), Span::styled("  navigation  ", dim),
            Span::styled("g", key),   Span::styled("  guidance   ", dim),
            Span::styled("r", key),   Span::styled("  routing    ", dim),
            Span::styled("m", key),   Span::styled("  map-matching", dim),
        ]),
        Line::from(vec![
            Span::styled("  ↑↓", key), Span::styled(" scroll      ", dim),
            Span::styled("f", key),    Span::styled("  follow     ", dim),
            Span::styled("q", key),    Span::styled("  quit", dim),
        ]),
        Line::from(vec![
            Span::styled("  PgUp/Dn", key), Span::styled(" page        ", dim),
            Span::styled("/", key),          Span::styled("  search     ", dim),
            Span::styled("0", key),          Span::styled("  all on     ", dim),
            Span::styled("-", key),          Span::styled("  all off    ", dim),
            Span::styled("?", key),          Span::styled("  help", dim),
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
