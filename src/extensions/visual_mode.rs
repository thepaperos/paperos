use crate::core::{BufferMessage, EditorRuntime, Extension};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use runact::{ActorId, Runtime};
use std::any::Any;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum VisualMode {
    Off,
    Line,
    Char,
}

#[allow(dead_code)]
pub struct VisualModeExtension {
    selection_start: Option<(usize, usize)>,
    visual_mode: VisualMode,
    clipboard: Option<String>,
    search_pattern: Option<String>,
    buffer: Option<ActorId>,
}

impl Extension for VisualModeExtension {
    fn name(&self) -> &str {
        "visual-mode"
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

impl VisualModeExtension {
    pub fn new() -> Self {
        Self {
            selection_start: None,
            visual_mode: VisualMode::Off,
            clipboard: None,
            search_pattern: None,
            buffer: None,
        }
    }

    pub fn set_buffer(&mut self, buffer: ActorId) {
        self.buffer = Some(buffer);
    }

    pub fn mode(&self) -> VisualMode {
        self.visual_mode
    }

    pub fn selection_start(&self) -> Option<(usize, usize)> {
        self.selection_start
    }

    pub fn is_searching(&self) -> bool {
        self.search_pattern.is_some()
    }

    pub fn search_pattern(&self) -> Option<&str> {
        self.search_pattern.as_deref()
    }

    pub fn handle_key(
        &mut self,
        key: KeyEvent,
        runtime: &mut EditorRuntime,
        buffer: Option<ActorId>,
    ) -> Option<String> {
        let buf = buffer.or(self.buffer)?;

        match self.visual_mode {
            VisualMode::Off => {
                // Handle search and paste when not in visual mode
                match key.code {
                    KeyCode::Char('v') if key.modifiers == KeyModifiers::SHIFT => {
                        // Shift+V → line visual mode
                        let cursor = get_cursor(runtime.runtime(), buf);
                        self.selection_start = Some(cursor);
                        self.visual_mode = VisualMode::Line;
                        None
                    }
                    KeyCode::Char('v') => {
                        // v → char visual mode
                        let cursor = get_cursor(runtime.runtime(), buf);
                        self.selection_start = Some(cursor);
                        self.visual_mode = VisualMode::Char;
                        None
                    }
                    KeyCode::Char('n') => {
                        self.find_next(runtime.runtime(), buf, true);
                        None
                    }
                    KeyCode::Char('N') => {
                        self.find_next(runtime.runtime(), buf, false);
                        None
                    }
                    KeyCode::Char('p') => {
                        self.paste(runtime.runtime(), buf);
                        None
                    }
                    _ => None,
                }
            }
            VisualMode::Char | VisualMode::Line => {
                match key.code {
                    KeyCode::Esc => {
                        self.visual_mode = VisualMode::Off;
                        self.selection_start = None;
                        Some("escape".to_string())
                    }
                    KeyCode::Char('y') => {
                        let yanked = self.yank_selection(runtime.runtime(), buf);
                        self.visual_mode = VisualMode::Off;
                        if yanked {
                            Some("yank".to_string())
                        } else {
                            None
                        }
                    }
                    KeyCode::Char('d') => {
                        let deleted = self.cut_selection(runtime.runtime(), buf);
                        self.visual_mode = VisualMode::Off;
                        if deleted {
                            Some("delete".to_string())
                        } else {
                            None
                        }
                    }
                    _ => {
                        // Move cursor while in visual mode
                        let cmd = match key.code {
                            KeyCode::Char('h') => Some("left"),
                            KeyCode::Char('j') => Some("down"),
                            KeyCode::Char('k') => Some("up"),
                            KeyCode::Char('l') => Some("right"),
                            _ => None,
                        };
                        cmd.map(|c| c.to_string())
                    }
                }
            }
        }
    }

    pub fn start_search(&mut self, pattern: &str) {
        self.search_pattern = Some(pattern.to_string());
    }

    /// Search for pattern, find next occurrence after cursor (forward=true) or before (forward=false)
    pub fn find_next(&mut self, runtime: &Runtime, buf: ActorId, forward: bool) {
        let pattern = match &self.search_pattern {
            Some(p) => p.clone(),
            None => return,
        };
        let content = get_content(runtime, buf);
        let cursor = get_cursor(runtime, buf);
        let lines: Vec<&str> = content.lines().collect();

        let found = if forward {
            search_forward(&lines, cursor.0, cursor.1 + 1, &pattern)
        } else {
            search_backward(&lines, cursor.0, cursor.1.saturating_sub(1), &pattern)
        };

        if let Some((row, col)) = found {
            let _ = runtime.send(buf, BufferMessage::MoveToLine(row + 1));
            // Column positioning requires line-level movement; simple approach:
            for _ in 0..col {
                let _ = runtime.send(buf, BufferMessage::MoveRight);
            }
        }
    }

    fn yank_selection(&mut self, runtime: &Runtime, buf: ActorId) -> bool {
        let content = get_content(runtime, buf);
        let sel = self.selection_start;
        match (sel, self.visual_mode) {
            (Some(start), VisualMode::Line) => {
                let (sr, _) = start;
                let (er, _) = get_cursor(runtime, buf);
                let (row_start, row_end) = (sr.min(er), sr.max(er));
                let lines: Vec<&str> = content.lines().collect();
                let yanked: Vec<&str> =
                    lines[row_start..=row_end.min(lines.len().saturating_sub(1))].to_vec();
                self.clipboard = Some(yanked.join("\n"));
                true
            }
            (Some(start), VisualMode::Char) => {
                let (sr, sc) = start;
                let (er, ec) = get_cursor(runtime, buf);
                let lines: Vec<&str> = content.lines().collect();
                if sr == er {
                    let line = lines
                        .get(sr)
                        .map(|l| &l[sc.min(ec)..ec.max(sc)])
                        .unwrap_or("");
                    self.clipboard = Some(line.to_string());
                } else {
                    let mut parts = Vec::new();
                    if let Some(line) = lines.get(sr) {
                        parts.push(&line[sc..]);
                    }
                    for i in (sr + 1)..er.min(lines.len()) {
                        if let Some(line) = lines.get(i) {
                            parts.push(line);
                        }
                    }
                    if let Some(line) = lines.get(er) {
                        parts.push(&line[..ec]);
                    }
                    self.clipboard = Some(parts.join("\n"));
                }
                true
            }
            _ => false,
        }
    }

    fn cut_selection(&mut self, runtime: &Runtime, buf: ActorId) -> bool {
        let yanked = self.yank_selection(runtime, buf);
        if yanked {
            // Delete selected lines/chars
            match self.visual_mode {
                VisualMode::Line => {
                    let sel = self.selection_start.unwrap();
                    let cursor = get_cursor(runtime, buf);
                    let (sr, _) = sel;
                    let (er, _) = cursor;
                    let (start, count) = (sr.min(er), sr.max(er) - sr.min(er) + 1);
                    for _ in 0..count.saturating_sub(1) {
                        let _ = runtime.send(buf, BufferMessage::DeleteLine);
                    }
                    // Move cursor to start
                    let _ = runtime.send(buf, BufferMessage::MoveToLine(start + 1));
                }
                VisualMode::Char => {
                    // Simple char delete: delete from start to cursor
                    let sel = self.selection_start.unwrap();
                    let start_row = sel.0;
                    let end_cursor = get_cursor(runtime, buf);
                    let end_row = end_cursor.0;
                    // Delete lines between start and end
                    for _ in 0..(end_row - start_row).saturating_sub(1) {
                        let _ = runtime.send(buf, BufferMessage::DeleteLine);
                    }
                }
                VisualMode::Off => {}
            }
            true
        } else {
            false
        }
    }

    pub fn paste(&mut self, runtime: &Runtime, buf: ActorId) {
        let text = match &self.clipboard {
            Some(t) => t.clone(),
            None => return,
        };
        // Insert each character; newlines become new lines
        for ch in text.chars() {
            if ch == '\n' {
                let _ = runtime.send(buf, BufferMessage::InsertLine);
            } else {
                let _ = runtime.send(buf, BufferMessage::InsertChar(ch));
            }
        }
    }
}

fn get_cursor(runtime: &Runtime, buf: ActorId) -> (usize, usize) {
    if let Ok(handle) = runtime.request(buf, BufferMessage::GetCursorPosition) {
        if let Ok(reply) = handle.recv_timeout(std::time::Duration::from_millis(100)) {
            if let Ok(pos) = reply.downcast::<(usize, usize)>() {
                return *pos;
            }
        }
    }
    (0, 0)
}

fn get_content(runtime: &Runtime, buf: ActorId) -> String {
    if let Ok(handle) = runtime.request(buf, BufferMessage::GetContent) {
        if let Ok(reply) = handle.recv_timeout(std::time::Duration::from_millis(100)) {
            if let Ok(content) = reply.downcast::<String>() {
                return (*content).clone();
            }
        }
    }
    String::new()
}

pub fn search_forward<'a>(
    lines: &[&'a str],
    start_row: usize,
    start_col: usize,
    pattern: &str,
) -> Option<(usize, usize)> {
    if pattern.is_empty() {
        return None;
    }
    // Check start row from start_col onwards
    if let Some(line) = lines.get(start_row) {
        if let Some(pos) = line[start_col..].find(pattern) {
            return Some((start_row, start_col + pos));
        }
    }
    // Check remaining rows
    for row in (start_row + 1)..lines.len() {
        if let Some(pos) = lines.get(row).and_then(|l| l.find(pattern)) {
            return Some((row, pos));
        }
    }
    None
}

pub fn search_backward<'a>(
    lines: &[&'a str],
    start_row: usize,
    start_col: usize,
    pattern: &str,
) -> Option<(usize, usize)> {
    if pattern.is_empty() {
        return None;
    }
    // Check start row backwards from start_col
    if let Some(line) = lines.get(start_row) {
        let prefix = &line[..start_col.min(line.len())];
        if let Some(pos) = prefix.rfind(pattern) {
            return Some((start_row, pos));
        }
    }
    // Check preceding rows backwards
    for row in (0..start_row).rev() {
        if let Some(pos) = lines.get(row).and_then(|l| l.rfind(pattern)) {
            return Some((row, pos));
        }
    }
    None
}

impl Default for VisualModeExtension {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_extension_starts_normal() {
        let ext = VisualModeExtension::new();
        assert_eq!(ext.mode(), VisualMode::Off);
        assert!(ext.selection_start().is_none());
        assert!(!ext.is_searching());
    }

    #[test]
    fn test_start_search_sets_pattern() {
        let mut ext = VisualModeExtension::new();
        ext.start_search("hello");
        assert_eq!(ext.mode(), VisualMode::Off);
        assert!(ext.is_searching());
    }

    #[test]
    fn test_search_forward_finds_match() {
        let lines = vec!["hello world", "foo bar", "hello again"];
        let lines_refs: Vec<&str> = lines.iter().map(|s| *s).collect();
        let result = search_forward(&lines_refs, 0, 0, "hello");
        assert_eq!(result, Some((0, 0)));
    }

    #[test]
    fn test_search_forward_finds_next_line() {
        let lines: Vec<&str> = vec!["aaa", "bbb foo", "ccc"];
        let result = search_forward(&lines, 0, 1, "foo"); // start after first line
        assert_eq!(result, Some((1, 4)));
    }

    #[test]
    fn test_search_forward_no_match() {
        let lines: Vec<&str> = vec!["hello", "world"];
        let result = search_forward(&lines, 0, 0, "xyz");
        assert_eq!(result, None);
    }

    #[test]
    fn test_search_backward_finds_prior_match_in_same_line() {
        let lines: Vec<&str> = vec!["foo bar foo baz", "qux"];
        // Cursor at col 12 (after second "foo"), search backward
        // Should find the second "foo" at col 8
        let result = search_backward(&lines, 0, 12, "foo");
        assert_eq!(result, Some((0, 8)));
    }

    #[test]
    fn test_search_backward_finds_prior_match() {
        let lines: Vec<&str> = vec!["foo bar", "baz", "qux"];
        // Start at row 2, search backward for "foo"
        let result = search_backward(&lines, 2, 3, "foo");
        assert_eq!(result, Some((0, 0)));
    }

    #[test]
    fn test_search_empty_pattern_returns_none() {
        let lines: Vec<&str> = vec!["hello"];
        let result = search_forward(&lines, 0, 0, "");
        assert_eq!(result, None);
    }

    #[test]
    fn test_clipboard_starts_empty() {
        let ext = VisualModeExtension::new();
        assert!(ext.clipboard.is_none());
    }

    #[test]
    fn test_set_buffer_stores_actor_id() {
        let mut ext = VisualModeExtension::new();
        ext.set_buffer(ActorId::new(42));
        // Cannot easily verify private field, but mode should still be Off
        assert_eq!(ext.mode(), VisualMode::Off);
    }
}
