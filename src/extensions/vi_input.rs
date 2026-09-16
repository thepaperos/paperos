use crate::core::Extension;
use runact::Runtime;
use std::any::Any;

/// ViInputExtension: vi-style keybindings (h/j/k/l movement, modes).
/// For basic/text-editor keybindings, use InputExtension instead.
#[allow(dead_code)]
pub struct ViInputExtension {
    mode: EditorMode,
    command_buffer: String,
    last_command: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Normal,
    Insert,
    Command,
    Search,
}

impl Extension for ViInputExtension {
    fn name(&self) -> &str {
        "vi-input"
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
impl ViInputExtension {
    pub fn new() -> Self {
        Self {
            mode: EditorMode::Normal,
            command_buffer: String::new(),
            last_command: None,
        }
    }

    pub fn mode(&self) -> EditorMode {
        self.mode
    }

    pub fn command_buffer(&self) -> &str {
        &self.command_buffer
    }

    /// Translate a character key press into a command string.
    pub fn handle_char_key(&mut self, ch: char) -> Option<String> {
        match self.mode {
            EditorMode::Insert => {
                if ch == '\x1b' {
                    self.mode = EditorMode::Normal;
                    None
                } else if ch == '\x08' || ch == '\x7f' {
                    Some("delete".to_string())
                } else if ch == '\r' || ch == '\n' {
                    Some("newline".to_string())
                } else if ch.is_control() {
                    None
                } else {
                    Some(format!("insert {}", ch))
                }
            }
            EditorMode::Normal => match ch {
                'i' => {
                    self.mode = EditorMode::Insert;
                    None
                }
                'h' => Some("left".to_string()),
                'j' => Some("down".to_string()),
                'k' => Some("up".to_string()),
                'l' => Some("right".to_string()),
                'w' => Some("w".to_string()),
                'b' => Some("b".to_string()),
                'e' => Some("e".to_string()),
                'G' => Some("lastline".to_string()),
                'u' => Some("undo".to_string()),
                'U' => Some("redo".to_string()),
                '0' => Some("0".to_string()),
                '$' => Some("$".to_string()),
                'x' => Some("delete".to_string()),
                'd' => Some("deleteline".to_string()),
                ':' => {
                    self.mode = EditorMode::Command;
                    self.command_buffer.clear();
                    None
                }
                '/' => {
                    self.mode = EditorMode::Search;
                    self.command_buffer.clear();
                    None
                }
                _ => None,
            },
            EditorMode::Command => {
                if ch == '\r' || ch == '\n' {
                    let cmd = self.command_buffer.clone();
                    self.command_buffer.clear();
                    self.mode = EditorMode::Normal;
                    self.last_command = Some(cmd.clone());
                    Some(cmd)
                } else if ch == '\x08' || ch == '\x7f' {
                    self.command_buffer.pop();
                    None
                } else if ch == '\x1b' {
                    self.command_buffer.clear();
                    self.mode = EditorMode::Normal;
                    None
                } else if ch.is_control() {
                    None
                } else {
                    self.command_buffer.push(ch);
                    None
                }
            }
            EditorMode::Search => {
                if ch == '\r' || ch == '\n' {
                    let pattern = self.command_buffer.clone();
                    self.command_buffer.clear();
                    self.mode = EditorMode::Normal;
                    self.last_command = Some(pattern.clone());
                    Some(format!("search {}", pattern))
                } else if ch == '\x08' || ch == '\x7f' {
                    self.command_buffer.pop();
                    None
                } else if ch == '\x1b' {
                    self.command_buffer.clear();
                    self.mode = EditorMode::Normal;
                    None
                } else if ch.is_control() {
                    None
                } else {
                    self.command_buffer.push(ch);
                    None
                }
            }
        }
    }

    /// Set the current editor mode.
    pub fn set_mode(&mut self, mode: EditorMode) {
        self.mode = mode;
    }

    pub fn take_last_command(&mut self) -> Option<String> {
        self.last_command.take()
    }
}

impl Default for ViInputExtension {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_mode_escape_returns_normal() {
        let mut ext = ViInputExtension::new();
        ext.handle_char_key('i');
        assert_eq!(ext.mode(), EditorMode::Insert);
        let cmd = ext.handle_char_key('\x1b');
        assert!(cmd.is_none());
        assert_eq!(ext.mode(), EditorMode::Normal);
    }

    #[test]
    fn test_normal_mode_hjkl_commands() {
        let mut ext = ViInputExtension::new();
        assert_eq!(ext.handle_char_key('h'), Some("left".to_string()));
        assert_eq!(ext.handle_char_key('j'), Some("down".to_string()));
        assert_eq!(ext.handle_char_key('k'), Some("up".to_string()));
        assert_eq!(ext.handle_char_key('l'), Some("right".to_string()));
    }

    #[test]
    fn test_insert_mode_characters() {
        let mut ext = ViInputExtension::new();
        ext.handle_char_key('i');
        assert_eq!(ext.handle_char_key('a'), Some("insert a".to_string()));
        assert_eq!(ext.handle_char_key('b'), Some("insert b".to_string()));
        assert_eq!(ext.handle_char_key('x'), Some("insert x".to_string()));
    }

    #[test]
    fn test_insert_mode_newline() {
        let mut ext = ViInputExtension::new();
        ext.handle_char_key('i');
        assert_eq!(ext.handle_char_key('\r'), Some("newline".to_string()));
        assert_eq!(ext.handle_char_key('\n'), Some("newline".to_string()));
    }

    #[test]
    fn test_insert_mode_backspace() {
        let mut ext = ViInputExtension::new();
        ext.handle_char_key('i');
        assert_eq!(ext.handle_char_key('\x08'), Some("delete".to_string()));
        assert_eq!(ext.handle_char_key('\x7f'), Some("delete".to_string()));
    }

    #[test]
    fn test_command_mode() {
        let mut ext = ViInputExtension::new();
        assert!(ext.handle_char_key(':').is_none());
        assert_eq!(ext.mode(), EditorMode::Command);
        ext.handle_char_key('w');
        let cmd = ext.handle_char_key('\r');
        assert_eq!(cmd, Some("w".to_string()));
        assert_eq!(ext.mode(), EditorMode::Normal);
    }

    #[test]
    fn test_command_mode_escape_cancels() {
        let mut ext = ViInputExtension::new();
        ext.handle_char_key(':');
        ext.handle_char_key('x');
        ext.handle_char_key('\x1b');
        assert_eq!(ext.mode(), EditorMode::Normal);
    }

    #[test]
    fn test_command_mode_backspace() {
        let mut ext = ViInputExtension::new();
        ext.handle_char_key(':');
        ext.handle_char_key('w');
        ext.handle_char_key('r');
        ext.handle_char_key('\x08');
        assert_eq!(ext.command_buffer(), "w");
    }

    #[test]
    fn test_search_mode_enter_and_execute() {
        let mut ext = ViInputExtension::new();
        assert!(ext.handle_char_key('/').is_none());
        assert_eq!(ext.mode(), EditorMode::Search);
        ext.handle_char_key('f');
        ext.handle_char_key('o');
        ext.handle_char_key('o');
        let cmd = ext.handle_char_key('\r');
        assert_eq!(cmd, Some("search foo".to_string()));
        assert_eq!(ext.mode(), EditorMode::Normal);
    }

    #[test]
    fn test_search_mode_escape_cancels() {
        let mut ext = ViInputExtension::new();
        ext.handle_char_key('/');
        ext.handle_char_key('a');
        ext.handle_char_key('\x1b');
        assert_eq!(ext.mode(), EditorMode::Normal);
    }

    #[test]
    fn test_search_mode_backspace() {
        let mut ext = ViInputExtension::new();
        ext.handle_char_key('/');
        ext.handle_char_key('a');
        ext.handle_char_key('b');
        ext.handle_char_key('\x08');
        assert_eq!(ext.command_buffer(), "a");
    }

    #[test]
    fn test_take_last_command() {
        let mut ext = ViInputExtension::new();
        ext.handle_char_key(':');
        ext.handle_char_key('w');
        ext.handle_char_key('\r');
        assert_eq!(ext.take_last_command(), Some("w".to_string()));
        assert_eq!(ext.take_last_command(), None);
    }

    #[test]
    fn test_control_chars_ignored_in_normal_mode() {
        let mut ext = ViInputExtension::new();
        assert_eq!(ext.handle_char_key('\x01'), None);
        assert_eq!(ext.handle_char_key('\x1b'), None);
    }
}
