//! Acceptance tests for viewport scrolling.
//!
//! Criterion: the editor viewport must scroll with the real terminal's
//! text-area size (variable height AND width), the cursor must never end up
//! hidden below the visible area (which previously clipped content right
//! above the status bar), and scroll offsets must be clamped so content can
//! never scroll past the end of the document.
//!
//! These hit the real entry points: RenderExtension::ensure_cursor_visible,
//! RenderExtension::set_viewport and RenderExtension::clamp_scroll.

use paperos::extensions::RenderExtension;

/// A viewport the size of a typical small terminal text area (minus border).
const ROWS: usize = 24;
const COLS: usize = 80;

fn ext_with_viewport(rows: usize, cols: usize) -> RenderExtension {
    let ext = RenderExtension::new();
    ext.set_viewport(rows, cols);
    ext
}

// -- AC1: cursor moving below the visible bottom scrolls the viewport down --

#[test]
fn cursor_at_bottom_edge_scrolls_viewport_down() {
    let ext = ext_with_viewport(ROWS, COLS);

    // Cursor lands on the first row past the visible area (row index == ROWS).
    ext.ensure_cursor_visible(ROWS, 0);

    assert_eq!(ext.scroll_row(), 1);
    assert!(ROWS >= ext.scroll_row());
    assert!(ROWS - ext.scroll_row() < ROWS);
}

#[test]
fn cursor_deep_in_document_keeps_cursor_inside_viewport() {
    let ext = ext_with_viewport(ROWS, COLS);

    ext.ensure_cursor_visible(3 * ROWS, 0);

    // Cursor must be within the visible window: scroll_row <= row < scroll_row + ROWS.
    assert!(3 * ROWS >= ext.scroll_row());
    assert!(3 * ROWS < ext.scroll_row() + ROWS);
}

// -- AC2: cursor moving above the visible top scrolls the viewport back up --

#[test]
fn cursor_above_viewport_top_scrolls_back_up() {
    let ext = ext_with_viewport(ROWS, COLS);

    ext.ensure_cursor_visible(ROWS, 0);
    assert_eq!(ext.scroll_row(), 1);

    ext.ensure_cursor_visible(0, 0);
    assert_eq!(ext.scroll_row(), 0);
}

// -- AC3: horizontal scrolling adapts to the visible width --

#[test]
fn cursor_past_right_edge_scrolls_horizontally() {
    let ext = ext_with_viewport(ROWS, COLS);

    ext.ensure_cursor_visible(0, COLS);
    assert_eq!(ext.scroll_col(), 1);

    ext.ensure_cursor_visible(0, 0);
    assert_eq!(ext.scroll_col(), 0);
}

#[test]
fn horizontal_scroll_uses_narrow_viewport_width() {
    // A narrow window (e.g. 40 cols) must scroll long lines earlier than a wide one.
    let narrow = ext_with_viewport(ROWS, 40);
    let wide = ext_with_viewport(ROWS, 80);

    narrow.ensure_cursor_visible(0, 60);
    wide.ensure_cursor_visible(0, 60);

    assert!(narrow.scroll_col() > wide.scroll_col());
    assert_eq!(narrow.scroll_col(), 60 - (40 - 1));
    assert_eq!(wide.scroll_col(), 0); // 60 < 80: no horizontal scroll needed
}

// -- AC4: scroll offsets are clamped so the document can never be scrolled past --
// (This is what previously let content "hide" — the viewport could scroll the
// bottom of the page into the status bar area or a blank region.)

#[test]
fn vertical_scroll_clamped_at_document_end() {
    let ext = ext_with_viewport(ROWS, COLS);

    // Simulate a cursor way down in a 30-line document.
    ext.ensure_cursor_visible(2 * ROWS, 0);
    ext.clamp_scroll(30, 40);

    // Max scroll for 30 lines / 24 visible rows is 30 - 24 = 6.
    assert_eq!(ext.scroll_row(), 30 - ROWS);
    // The last document line stays visible above the status bar:
    assert_eq!(ext.scroll_row() + ROWS, 30);
}

#[test]
fn horizontal_scroll_clamped_at_longest_line() {
    let ext = ext_with_viewport(ROWS, COLS);

    ext.ensure_cursor_visible(0, 200);
    ext.clamp_scroll(30, 50);

    // Longest line is 50 cols, viewport is 80 wide -> no horizontal scroll.
    assert_eq!(ext.scroll_col(), 0);
}

#[test]
fn short_document_never_scrolls() {
    let ext = ext_with_viewport(ROWS, COLS);

    // 10-line document in a 24-row viewport: nothing to scroll.
    ext.ensure_cursor_visible(9, 0);
    ext.clamp_scroll(10, 30);
    assert_eq!(ext.scroll_row(), 0);
}

// -- AC5: various viewport sizes are honored (variable height & width) --

#[test]
fn very_tall_viewport_uses_its_own_height() {
    let ext = ext_with_viewport(50, 100);

    ext.ensure_cursor_visible(120, 0);
    // 120-line cursor in a 50-row viewport keeps cursor on the last visible row.
    assert_eq!(ext.scroll_row(), 120 - (50 - 1));
}

#[test]
fn very_short_viewport_uses_its_own_height() {
    let ext = ext_with_viewport(5, 20);

    ext.ensure_cursor_visible(20, 0);
    assert_eq!(ext.scroll_row(), 20 - (5 - 1));
}
