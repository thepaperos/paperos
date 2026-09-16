use crate::core::Extension;
use crossterm::event::KeyCode;
use runact::Runtime;
use std::any::Any;

/// InputExtension: default editor keybindings (basic/text-editor mode).
/// Arrow keys move cursor, typing inserts directly, no mode switching.
/// Vi-style keybindings are provided by ViInputExtension (see vi_input.rs).
#[allow(dead_code)]
pub struct InputExtension {
    active: bool,
}

#[allow(dead_code)]
impl InputExtension {
    pub fn new() -> Self {
        Self { active: true }
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Handle a key event.
    pub fn handle_key(&mut self, code: &KeyCode) -> Option<String> {
        if !self.active {
            return None;
        }

        match code {
            KeyCode::Left => Some("left".to_string()),
            KeyCode::Right => Some("right".to_string()),
            KeyCode::Up => Some("up".to_string()),
            KeyCode::Down => Some("down".to_string()),
            KeyCode::Enter => Some("newline".to_string()),
            KeyCode::Backspace => Some("delete".to_string()),
            KeyCode::Home => Some("0".to_string()),
            KeyCode::End => Some("$".to_string()),
            KeyCode::PageUp => Some("scrollup".to_string()),
            KeyCode::PageDown => Some("scrolldown".to_string()),
            KeyCode::Char(ch) => {
                if ch.is_control() {
                    None
                } else {
                    Some(format!("insert {}", ch))
                }
            }
            KeyCode::Tab => Some("insert \t".to_string()),
            _ => None,
        }
    }
}

impl Extension for InputExtension {
    fn name(&self) -> &str {
        "input"
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

impl Default for InputExtension {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_mode_arrow_keys() {
        let mut ext = InputExtension::new();
        assert!(ext.is_active());
        assert_eq!(ext.handle_key(&KeyCode::Left), Some("left".to_string()));
        assert_eq!(ext.handle_key(&KeyCode::Right), Some("right".to_string()));
        assert_eq!(ext.handle_key(&KeyCode::Up), Some("up".to_string()));
        assert_eq!(ext.handle_key(&KeyCode::Down), Some("down".to_string()));
    }

    #[test]
    fn test_basic_mode_typing_inserts() {
        let mut ext = InputExtension::new();
        assert_eq!(
            ext.handle_key(&KeyCode::Char('a')),
            Some("insert a".to_string())
        );
        assert_eq!(
            ext.handle_key(&KeyCode::Char('z')),
            Some("insert z".to_string())
        );
    }

    #[test]
    fn test_basic_mode_control_keys() {
        let mut ext = InputExtension::new();
        assert_eq!(ext.handle_key(&KeyCode::Enter), Some("newline".to_string()));
        assert_eq!(
            ext.handle_key(&KeyCode::Backspace),
            Some("delete".to_string())
        );
        assert_eq!(ext.handle_key(&KeyCode::Home), Some("0".to_string()));
        assert_eq!(ext.handle_key(&KeyCode::End), Some("$".to_string()));
    }

    #[test]
    fn test_basic_mode_inactive_returns_none() {
        let mut ext = InputExtension::new();
        ext.set_active(false);
        assert!(!ext.is_active());
        assert!(ext.handle_key(&KeyCode::Char('a')).is_none());
        assert!(ext.handle_key(&KeyCode::Left).is_none());
    }

    #[test]
    fn test_basic_mode_control_chars_ignored() {
        let mut ext = InputExtension::new();
        assert!(ext.handle_key(&KeyCode::Char('\x01')).is_none());
        assert!(ext.handle_key(&KeyCode::Char('\x1b')).is_none());
    }
}
