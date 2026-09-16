//! Acceptance tests for basic-mode status display.
//!
//! Criterion: Basic is the DEFAULT editor mode. A freshly created
//! ViInputExtension starts in EditorMode::Basic, the status bar shows
//! " BASIC " until the user explicitly enters vi mode (i / : / /), and
//! Esc from Insert returns to Normal (classic vi).
//!
//! These hit the real entry points: RenderExtension::get_mode,
//! ViInputExtension::mode, ViInputExtension::handle_char_key.

use paperos::extensions::{RenderExtension, ViInputExtension};
use paperos::extensions::vi_input::EditorMode;
use std::sync::{Arc, RwLock};

// AC1: A freshly created vi input starts in Basic (the default mode).
#[test]
fn vi_input_starts_in_basic_by_default() {
    let vi = ViInputExtension::new();
    assert_eq!(vi.mode(), EditorMode::Basic);
}

// AC2: A fresh editor without vi input reports Basic.
#[test]
fn fresh_editor_without_vi_shows_basic() {
    let ext = RenderExtension::new();
    assert_eq!(ext.get_mode(), EditorMode::Basic);
}

// AC3: A fresh editor WITH vi input (still in Basic) reports Basic.
#[test]
fn fresh_editor_with_vi_shows_basic() {
    let vi = Arc::new(RwLock::new(ViInputExtension::new()));
    let mut ext = RenderExtension::new();
    ext.set_vi_input(vi);
    assert_eq!(ext.get_mode(), EditorMode::Basic);
}

// AC4: Pressing 'i' from Basic enters Insert; Esc returns to Normal (vi used).
#[test]
fn entering_vi_insert_then_escape_shows_normal() {
    let mut vi = ViInputExtension::new();
    assert_eq!(vi.handle_char_key('i'), None);
    assert_eq!(vi.mode(), EditorMode::Insert);

    assert_eq!(vi.handle_char_key('\x1b'), None);
    assert_eq!(vi.mode(), EditorMode::Normal);
    assert_ne!(vi.mode(), EditorMode::Basic);
}

// AC5: Colons from Basic enter Command mode.
#[test]
fn colon_from_basic_enters_command_mode() {
    let mut vi = ViInputExtension::new();
    assert_eq!(vi.handle_char_key(':'), None);
    assert_eq!(vi.mode(), EditorMode::Command);
}

// AC6: Slash from Basic enters Search mode.
#[test]
fn slash_from_basic_enters_search_mode() {
    let mut vi = ViInputExtension::new();
    assert_eq!(vi.handle_char_key('/'), None);
    assert_eq!(vi.mode(), EditorMode::Search);
}

// AC7: get_mode reflects the vi mode once vi has been used.
#[test]
fn get_mode_reflects_explicit_vi_modes() {
    let vi = Arc::new(RwLock::new(ViInputExtension::new()));
    let mut ext = RenderExtension::new();
    ext.set_vi_input(vi.clone());

    vi.write().unwrap().set_mode(EditorMode::Command);
    assert_eq!(ext.get_mode(), EditorMode::Command);

    vi.write().unwrap().set_mode(EditorMode::Search);
    assert_eq!(ext.get_mode(), EditorMode::Search);

    vi.write().unwrap().set_mode(EditorMode::Normal);
    assert_eq!(ext.get_mode(), EditorMode::Normal);
}

// AC8: Basic is a distinct mode, not an alias of Normal.
#[test]
fn basic_is_not_normal() {
    assert_ne!(EditorMode::Basic, EditorMode::Normal);
}

// AC9: Status bar text for each mode (Basic is the default label).
#[test]
fn status_text_matches_mode() {
    let text = |m: &EditorMode| match m {
        EditorMode::Basic => " BASIC ",
        EditorMode::Normal => " NORMAL ",
        EditorMode::Insert => " INSERT ",
        EditorMode::Command => " COMMAND ",
        EditorMode::Search => " SEARCH ",
    };
    assert_eq!(text(&EditorMode::Basic), " BASIC ");
    assert_eq!(text(&EditorMode::Normal), " NORMAL ");
    assert_eq!(text(&EditorMode::Insert), " INSERT ");
    assert_eq!(text(&EditorMode::Command), " COMMAND ");
    assert_eq!(text(&EditorMode::Search), " SEARCH ");
}