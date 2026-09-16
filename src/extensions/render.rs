use super::basic_input::InputExtension;
use super::vi_input::{EditorMode, ViInputExtension};
use super::visual_mode::{VisualMode, VisualModeExtension};
use crate::core::{BufferMessage, EditorRuntime, Extension};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Terminal;
use runact::{ActorId, Runtime};
use std::any::Any;
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

pub struct RenderExtension {
    buffer: Option<ActorId>,
    input: Option<Arc<RwLock<InputExtension>>>,
    vi_input: Option<Arc<RwLock<ViInputExtension>>>,
    visual_mode: Option<Arc<RwLock<VisualModeExtension>>>,
    scroll_row: AtomicUsize,
    scroll_col: AtomicUsize,
    /// Size of the editor text area (rows x cols) from the last rendered
    /// frame — excludes the status bar and the paragraph border. Refreshed
    /// every draw so the viewport adapts to terminal resizes.
    viewport_rows: AtomicUsize,
    viewport_cols: AtomicUsize,
    show_line_numbers: bool,
}

impl Extension for RenderExtension {
    fn name(&self) -> &str {
        "render"
    }

    fn init(&mut self, _runtime: &mut Runtime) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn shutdown(&mut self, _runtime: &mut Runtime) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn on_key(&mut self, _ch: char) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn on_render(&mut self, _runtime: &Runtime) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[allow(dead_code)]
impl RenderExtension {
    pub fn new() -> Self {
        Self {
            buffer: None,
            input: None,
            vi_input: None,
            visual_mode: None,
            scroll_row: AtomicUsize::new(0),
            scroll_col: AtomicUsize::new(0),
            viewport_rows: AtomicUsize::new(24),
            viewport_cols: AtomicUsize::new(80),
            show_line_numbers: true,
        }
    }

    pub fn set_buffer(&mut self, buffer: ActorId) {
        self.buffer = Some(buffer);
    }

    pub fn set_input(&mut self, input: Arc<RwLock<InputExtension>>) {
        self.input = Some(input);
    }

    pub fn set_vi_input(&mut self, vi_input: Arc<RwLock<ViInputExtension>>) {
        self.vi_input = Some(vi_input);
    }

    /// Record the editor text-area size for the current frame.
    /// Called on every render with the actual frame dimensions so scrolling
    /// adapts to variable terminal height and width (including resizes).
    pub fn set_viewport(&self, rows: usize, cols: usize) {
        self.viewport_rows.store(rows.max(1), Ordering::Relaxed);
        self.viewport_cols.store(cols.max(1), Ordering::Relaxed);
    }

    pub fn scroll_row(&self) -> usize {
        self.scroll_row.load(Ordering::Relaxed)
    }

    pub fn scroll_col(&self) -> usize {
        self.scroll_col.load(Ordering::Relaxed)
    }

    /// Keep the scroll offsets inside the document: the viewport may never
    /// scroll the last line (or the longest line's end) out of view. Without
    /// this, scrolling past the end of the file leaves a blank page and the
    /// bottom of the document clips into the status bar area.
    pub fn clamp_scroll(&self, line_count: usize, max_line_len: usize) {
        let rows = self.viewport_rows.load(Ordering::Relaxed);
        let cols = self.viewport_cols.load(Ordering::Relaxed);
        let max_scroll_row = line_count.saturating_sub(rows);
        let sr = self.scroll_row.load(Ordering::Relaxed).min(max_scroll_row);
        self.scroll_row.store(sr, Ordering::Relaxed);
        let max_scroll_col = max_line_len.saturating_sub(cols);
        let sc = self.scroll_col.load(Ordering::Relaxed).min(max_scroll_col);
        self.scroll_col.store(sc, Ordering::Relaxed);
    }

    /// Auto-scroll horizontally and vertically to keep the cursor visible.
    /// Called after every key event so the cursor doesn't vanish off-screen
    /// when moving past the visible area or editing long lines.
    /// The visible area size comes from the last rendered frame, so it works
    /// for any terminal height and width.
    pub fn ensure_cursor_visible(&self, cursor_row: usize, cursor_col: usize) {
        let rows = self.viewport_rows.load(Ordering::Relaxed);
        let cols = self.viewport_cols.load(Ordering::Relaxed);

        // Vertical: scroll down if cursor is below visible area, up if above
        let scroll_row = self.scroll_row();
        if cursor_row < scroll_row {
            self.scroll_row.store(cursor_row, Ordering::Relaxed);
        } else if cursor_row >= scroll_row + rows {
            self.scroll_row
                .store(cursor_row.saturating_sub(rows - 1), Ordering::Relaxed);
        }

        // Horizontal: scroll right if cursor_col exceeds visible area
        let scroll_col = self.scroll_col();
        if cursor_col < scroll_col {
            self.scroll_col.store(cursor_col, Ordering::Relaxed);
        } else if cursor_col >= scroll_col + cols {
            self.scroll_col
                .store(cursor_col.saturating_sub(cols - 1), Ordering::Relaxed);
        }
    }

    /// Vim-style page scrolling: move the viewport AND the cursor by a page
    /// (or half page) in the given direction, so the cursor stays in view and
    /// the viewport is not snapped back on the next key press.
    fn page_scroll(
        &self,
        editor_runtime: &mut EditorRuntime,
        buffer: Option<ActorId>,
        dir: i64,
        full_page: bool,
    ) {
        let visible = self.viewport_rows.load(Ordering::Relaxed) as i64;
        let step = if full_page {
            visible
        } else {
            (visible / 2).max(1)
        };
        let step = step * dir;
        if step == 0 {
            return;
        }

        let sr = self.scroll_row() as i64;
        self.scroll_row
            .store((sr + step).max(0) as usize, Ordering::Relaxed);

        // Move the cursor by the same delta (clamped to the buffer) so it
        // remains inside the scrolled viewport.
        let cmd = if dir > 0 { "down" } else { "up" };
        for _ in 0..step.abs() {
            let _ = editor_runtime.run_command(cmd, buffer);
        }

        if let Some(buf) = buffer {
            if let Ok(handle) = editor_runtime
                .runtime()
                .request(buf, BufferMessage::GetCursorPosition)
            {
                if let Ok(reply) = handle.recv_timeout(std::time::Duration::from_millis(100)) {
                    if let Ok(pos) = reply.downcast::<(usize, usize)>() {
                        let (row, col) = *pos;
                        self.ensure_cursor_visible(row, col);
                    }
                }
            }
        }
    }

    pub fn set_visual_mode(&mut self, visual: Arc<RwLock<VisualModeExtension>>) {
        self.visual_mode = Some(visual);
    }

    pub fn run_event_loop(
        &self,
        editor_runtime: &mut EditorRuntime,
        buffer: Option<ActorId>,
    ) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|f| {
                self.render(f, editor_runtime.runtime());
            })?;

            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') && key.modifiers == KeyModifiers::CONTROL {
                        break;
                    }
                    if key.modifiers == KeyModifiers::CONTROL {
                        match key.code {
                            KeyCode::Char('d') => {
                                self.page_scroll(editor_runtime, buffer, 1, false);
                                continue;
                            }
                            KeyCode::Char('u') => {
                                self.page_scroll(editor_runtime, buffer, -1, false);
                                continue;
                            }
                            KeyCode::Char('f') => {
                                self.page_scroll(editor_runtime, buffer, 1, true);
                                continue;
                            }
                            KeyCode::Char('b') => {
                                self.page_scroll(editor_runtime, buffer, -1, true);
                                continue;
                            }
                            KeyCode::Left => {
                                let prev = self.scroll_col.load(Ordering::Relaxed);
                                self.scroll_col
                                    .store(prev.saturating_sub(1), Ordering::Relaxed);
                                continue;
                            }
                            KeyCode::Right => {
                                self.scroll_col.fetch_add(1, Ordering::Relaxed);
                                continue;
                            }
                            _ => {}
                        }
                    }
                    self.handle_key(key, editor_runtime, buffer);
                    // After handling, auto-scroll to keep cursor visible
                    if let Some(buf) = buffer {
                        if let Ok(handle) = editor_runtime
                            .runtime()
                            .request(buf, BufferMessage::GetCursorPosition)
                        {
                            if let Ok(reply) =
                                handle.recv_timeout(std::time::Duration::from_millis(100))
                            {
                                if let Ok(pos) = reply.downcast::<(usize, usize)>() {
                                    let (row, col) = *pos;
                                    self.ensure_cursor_visible(row, col);
                                }
                            }
                        }
                    }
                }
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        Ok(())
    }

    fn handle_key(
        &self,
        key: KeyEvent,
        editor_runtime: &mut EditorRuntime,
        buffer: Option<ActorId>,
    ) {
        // First check visual mode — even when Off, it may intercept v/V/n/N/p keys
        if let Some(ref visual) = self.visual_mode {
            if let Ok(mut vm_guard) = visual.write() {
                let was_off = vm_guard.mode() == VisualMode::Off;
                if let Some(cmd) = vm_guard.handle_key(key, editor_runtime, buffer) {
                    let _ = editor_runtime.run_command(&cmd, buffer);
                }
                // If visual mode changed, we consumed the key
                if vm_guard.mode() != VisualMode::Off || !was_off {
                    return;
                }
            }
        }

        let mut handled = false;

        // Check basic input mode (arrow keys, direct typing)
        if let Some(ref input) = self.input {
            let command = {
                if let Ok(mut input_guard) = input.write() {
                    input_guard.handle_key(&key.code)
                } else {
                    None
                }
            };
            if let Some(cmd) = command {
                if cmd.starts_with("search ") {
                    // Handle search command — pass pattern to VisualModeExtension
                    if let Some(ref visual) = self.visual_mode {
                        if let Some(buf) = buffer {
                            if let Ok(mut vm_guard) = visual.write() {
                                let pattern = cmd.strip_prefix("search ").unwrap_or("");
                                vm_guard.start_search(pattern);
                                vm_guard.find_next(editor_runtime.runtime(), buf, true);
                            }
                        }
                    }
                    handled = true;
                } else {
                    let _ = editor_runtime.run_command(&cmd, buffer);
                    handled = true;
                }
            }
        }

        // If basic input didn't handle it, try vi input
        if !handled {
            if let Some(ref vi) = self.vi_input {
                if let Some(ch) = key_to_char(&key) {
                    let command = {
                        if let Ok(mut vi_guard) = vi.write() {
                            vi_guard.handle_char_key(ch)
                        } else {
                            None
                        }
                    };
                    if let Some(cmd) = command {
                        if cmd.starts_with("search ") {
                            if let Some(ref visual) = self.visual_mode {
                                if let Some(buf) = buffer {
                                    if let Ok(mut vm_guard) = visual.write() {
                                        let pattern = cmd.strip_prefix("search ").unwrap_or("");
                                        vm_guard.start_search(pattern);
                                        vm_guard.find_next(editor_runtime.runtime(), buf, true);
                                    }
                                }
                            }
                        } else {
                            let _ = editor_runtime.run_command(&cmd, buffer);
                        }
                    }
                }
            }
        }
    }

    fn render(&self, f: &mut ratatui::Frame, runtime: &Runtime) {
        let content = self.get_buffer_content(runtime);
        let (cursor_row, cursor_col) = self.get_cursor_position(runtime);
        let mode = self.get_mode();
        let file_name = self.get_file_name(runtime);

        let area = f.area();

        // Layout: editor | status
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);

        let editor_area = vertical[0];

        // Build content lines, optionally prepending line numbers
        let max_line = content.lines().count().max(1);
        let num_digits = max_line.to_string().len();
        let num_width = if self.show_line_numbers {
            num_digits + 1 // digits + space separator
        } else {
            0
        };

        let inner_rows = editor_area.height.saturating_sub(2) as usize;
        let inner_cols = editor_area.width.saturating_sub(2) as usize;
        let scroll_col_now = self.scroll_col.load(Ordering::Relaxed);
        let gutter = if scroll_col_now == 0 {
            num_width as usize
        } else {
            0
        };
        let visible_cols = inner_cols.saturating_sub(gutter).max(1);
        self.set_viewport(inner_rows.max(1), visible_cols);

        self.clamp_scroll(
            max_line,
            content
                .lines()
                .map(|l| l.chars().count())
                .max()
                .unwrap_or(0),
        );

        let scroll_row = self.scroll_row.load(Ordering::Relaxed) as u16;
        let scroll_col = self.scroll_col.load(Ordering::Relaxed) as u16;
        let search_pattern = self.get_search_pattern();
        let lines: Vec<Line> = content
            .lines()
            .enumerate()
            .map(|(i, l)| {
                let text_spans = if search_pattern.is_some() {
                    self.highlight_search(l, search_pattern.as_deref(), cursor_row, i)
                } else {
                    vec![Span::raw(l.to_string())]
                };
                if self.show_line_numbers {
                    let padded = format!("{:>width$} ", i + 1, width = num_digits);
                    let mut spans =
                        vec![Span::styled(padded, Style::default().fg(Color::DarkGray))];
                    spans.extend(text_spans);
                    Line::from(spans)
                } else {
                    Line::from(text_spans)
                }
            })
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(format!(" paperOS — {} ", file_name))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: false })
            .scroll((scroll_row, scroll_col + num_width as u16));
        f.render_widget(paragraph, editor_area);

        // Cursor position: inside the editor area, with border offset + line number width
        let inner_x =
            editor_area.x + 1 + num_width as u16 + (cursor_col as u16).saturating_sub(scroll_col);
        let inner_y = editor_area.y + 1 + (cursor_row as u16).saturating_sub(scroll_row);

        let mode_text = match self.get_visual_mode() {
            // Vi mode with an active selection — both char and line visual show VISUAL.
            VisualMode::Char | VisualMode::Line => " VISUAL ",
            VisualMode::Off => match mode {
                EditorMode::Basic => " BASIC ",
                EditorMode::Normal => " NORMAL ",
                EditorMode::Insert => " INSERT ",
                EditorMode::Command => " COMMAND ",
                EditorMode::Search => " SEARCH ",
            },
        };
        let cmd_buf = self.command_buffer_display();
        let status = Paragraph::new(Line::from(vec![
            Span::styled(mode_text, Style::default().fg(Color::Black).bg(Color::Cyan)),
            Span::raw(" | "),
            Span::raw(&cmd_buf),
            Span::raw(" | "),
            Span::raw(format!("{}:{}", cursor_row + 1, cursor_col + 1)),
            Span::raw(" | Ctrl+Q: quit | /: search | n/N: find"),
        ]));
        f.render_widget(status, vertical[1]);
        f.set_cursor_position((inner_x, inner_y));
    }

    fn get_buffer_content(&self, runtime: &Runtime) -> String {
        if let Some(buffer_id) = self.buffer {
            if let Ok(handle) = runtime.request(buffer_id, BufferMessage::GetContent) {
                if let Ok(reply) = handle.recv_timeout(std::time::Duration::from_millis(100)) {
                    if let Ok(content) = reply.downcast::<String>() {
                        return (*content).clone();
                    }
                }
            }
        }
        String::new()
    }

    fn get_cursor_position(&self, runtime: &Runtime) -> (usize, usize) {
        if let Some(buffer_id) = self.buffer {
            if let Ok(handle) = runtime.request(buffer_id, BufferMessage::GetCursorPosition) {
                if let Ok(reply) = handle.recv_timeout(std::time::Duration::from_millis(100)) {
                    if let Ok(pos) = reply.downcast::<(usize, usize)>() {
                        return *pos;
                    }
                }
            }
        }
        (0, 0)
    }

    pub fn get_mode(&self) -> EditorMode {
        if let Some(ref vi) = self.vi_input {
            if let Ok(guard) = vi.read() {
                return guard.mode();
            }
        }
        // No vi input: the editor runs in basic mode by default.
        EditorMode::Basic
    }

    fn get_visual_mode(&self) -> VisualMode {
        if let Some(ref visual) = self.visual_mode {
            if let Ok(vm_guard) = visual.read() {
                return vm_guard.mode();
            }
        }
        VisualMode::Off
    }

    fn get_file_name(&self, runtime: &Runtime) -> String {
        if let Some(buffer_id) = self.buffer {
            if let Ok(handle) = runtime.request(buffer_id, BufferMessage::GetFilePath) {
                if let Ok(reply) = handle.recv_timeout(std::time::Duration::from_millis(100)) {
                    if let Ok(path) = reply.downcast::<Option<String>>() {
                        if let Some(ref p) = *path {
                            if let Some(name) = std::path::Path::new(p).file_name() {
                                return name.to_string_lossy().to_string();
                            }
                        }
                    }
                }
            }
        }
        "unnamed".to_string()
    }

    fn command_buffer_display(&self) -> String {
        // Check vi input for command/search buffers
        if let Some(ref vi) = self.vi_input {
            if let Ok(vi_guard) = vi.read() {
                if vi_guard.mode() == EditorMode::Command {
                    return format!(":{}", vi_guard.command_buffer());
                }
                if vi_guard.mode() == EditorMode::Search {
                    return format!("/{}", vi_guard.command_buffer());
                }
            }
        }
        String::new()
    }

    fn get_search_pattern(&self) -> Option<String> {
        if let Some(ref visual) = self.visual_mode {
            if let Ok(vm_guard) = visual.read() {
                return vm_guard.search_pattern().map(|s| s.to_string());
            }
        }
        None
    }

    /// Split a line's text into spans, highlighting the search pattern.
    /// The occurrence at `cursor_row == row_idx` is highlighted with the cursor color;
    /// all other occurrences get a subtler highlight.
    fn highlight_search(
        &self,
        text: &str,
        pattern: Option<&str>,
        cursor_row: usize,
        row_idx: usize,
    ) -> Vec<Span<'static>> {
        let pat = match pattern {
            Some(p) if !p.is_empty() => p,
            _ => return vec![Span::raw(text.to_string())],
        };
        let mut spans = Vec::new();
        let mut remaining = text;
        let mut offset = 0usize;
        while let Some(pos) = remaining.find(pat) {
            if pos > 0 {
                spans.push(Span::raw(remaining[..pos].to_string()));
                offset += pos;
            }
            let highlight = if row_idx == cursor_row {
                // Current match under cursor — use yellow background
                Span::styled(
                    remaining[pos..pos + pat.len()].to_string(),
                    Style::default().fg(Color::Black).bg(Color::Yellow),
                )
            } else {
                // Other matches — subtle yellow foreground
                Span::styled(
                    remaining[pos..pos + pat.len()].to_string(),
                    Style::default().fg(Color::Yellow),
                )
            };
            spans.push(highlight);
            remaining = &remaining[pos + pat.len()..];
            offset += pat.len();
        }
        if offset < text.len() {
            spans.push(Span::raw(remaining.to_string()));
        }
        if spans.is_empty() {
            spans.push(Span::raw(text.to_string()));
        }
        spans
    }
}

/// Convert a KeyEvent to a char for InputExtension handle_key.
/// Returns None for keys that aren't simple character inputs.
fn key_to_char(key: &KeyEvent) -> Option<char> {
    match key.code {
        KeyCode::Char(ch) => Some(ch),
        KeyCode::Esc => Some('\x1b'),
        KeyCode::Enter => Some('\r'),
        KeyCode::Backspace => Some('\x08'),
        KeyCode::Tab => Some('\t'),
        _ => None,
    }
}

impl Default for RenderExtension {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_search_no_pattern_returns_plain() {
        let ext = RenderExtension::new();
        let result = ext.highlight_search("hello world", None, 0, 0);
        assert_eq!(result.len(), 1);
        // Span content should be the full text
    }

    #[test]
    fn test_highlight_search_highlights_pattern() {
        let ext = RenderExtension::new();
        let result = ext.highlight_search("find the pattern here", Some("pattern"), 0, 0);
        // Should have: "find the ", highlighted "pattern", " here"
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_highlight_search_multiple_matches() {
        let ext = RenderExtension::new();
        let result = ext.highlight_search("foo bar foo", Some("foo"), 0, 0);
        // Should have: highlighted "foo", " bar ", highlighted "foo"
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_highlight_search_empty_pattern_returns_plain() {
        let ext = RenderExtension::new();
        let result = ext.highlight_search("hello", Some(""), 0, 0);
        assert_eq!(result.len(), 1);
    }
}
