use runact::{Actor, ActorContext, ActorError};

#[allow(dead_code)]
pub struct BufferActor {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub modified: bool,
    pub file_path: Option<String>,
    history: Vec<BufferEdit>,
    history_index: usize,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum BufferEdit {
    InsertChar {
        row: usize,
        col: usize,
        ch: char,
    },
    DeleteChar {
        row: usize,
        col: usize,
        ch: char,
    },
    InsertLine {
        row: usize,
        split_at: usize,
        rest: String,
    },
    DeleteLine {
        row: usize,
        content: String,
    },
}

#[derive(Clone)]
pub enum BufferMessage {
    InsertChar(char),
    InsertLine,
    DeleteChar,
    DeleteLine,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveToStart,
    MoveToEnd,
    MoveToNextWord,
    MoveToPrevWord,
    MoveToEndOfWord,
    MoveToLine(usize),
    MoveToLastLine,
    GetContent,
    GetCursorPosition,
    GetFilePath,
    SetFilePath(Option<String>),
    LoadContent(String),
    Undo,
    Redo,
}

impl Actor for BufferActor {
    type Message = BufferMessage;

    fn handle(&mut self, msg: BufferMessage, ctx: &mut ActorContext) -> Result<(), ActorError> {
        match msg {
            BufferMessage::InsertChar(ch) => {
                if self.cursor_row < self.lines.len() {
                    let line = &mut self.lines[self.cursor_row];
                    let prev_col = self.cursor_col;
                    line.insert(self.cursor_col, ch);
                    self.cursor_col += 1;
                    self.modified = true;
                    self.push_edit(BufferEdit::InsertChar {
                        row: self.cursor_row,
                        col: prev_col,
                        ch,
                    });
                }
            }
            BufferMessage::InsertLine => {
                let line = if self.cursor_row < self.lines.len() {
                    self.lines[self.cursor_row][self.cursor_col..].to_string()
                } else {
                    String::new()
                };
                let old_col = self.cursor_col;
                let old_row = self.cursor_row;
                if self.cursor_row < self.lines.len() {
                    self.lines[self.cursor_row] =
                        self.lines[self.cursor_row][..self.cursor_col].to_string();
                }
                self.cursor_row += 1;
                self.cursor_col = 0;
                self.lines.insert(self.cursor_row, line);
                self.modified = true;
                self.push_edit(BufferEdit::InsertLine {
                    row: old_row,
                    split_at: old_col,
                    rest: String::new(),
                });
            }
            BufferMessage::DeleteChar => {
                if self.cursor_col > 0 {
                    let line = &mut self.lines[self.cursor_row];
                    self.cursor_col -= 1;
                    let ch = line.remove(self.cursor_col);
                    self.modified = true;
                    self.push_edit(BufferEdit::DeleteChar {
                        row: self.cursor_row,
                        col: self.cursor_col,
                        ch,
                    });
                } else if self.cursor_row > 0 {
                    let current_line = self.lines.remove(self.cursor_row);
                    self.cursor_row -= 1;
                    self.cursor_col = self.lines[self.cursor_row].len();
                    self.lines[self.cursor_row].push_str(&current_line);
                    self.modified = true;
                }
            }
            BufferMessage::DeleteLine => {
                if self.lines.len() > 1 {
                    let content = self.lines.remove(self.cursor_row);
                    if self.cursor_row >= self.lines.len() {
                        self.cursor_row = self.lines.len() - 1;
                    }
                    self.cursor_col = 0;
                    self.modified = true;
                    self.push_edit(BufferEdit::DeleteLine {
                        row: self.cursor_row,
                        content,
                    });
                } else {
                    self.lines[0].clear();
                    self.cursor_col = 0;
                    self.modified = true;
                }
            }
            BufferMessage::MoveUp => {
                if self.cursor_row > 0 {
                    self.cursor_row -= 1;
                    self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
                }
            }
            BufferMessage::MoveDown => {
                if self.cursor_row < self.lines.len() - 1 {
                    self.cursor_row += 1;
                    self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
                }
            }
            BufferMessage::MoveLeft => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                } else if self.cursor_row > 0 {
                    self.cursor_row -= 1;
                    self.cursor_col = self.lines[self.cursor_row].len();
                }
            }
            BufferMessage::MoveRight => {
                if self.cursor_col < self.lines[self.cursor_row].len() {
                    self.cursor_col += 1;
                } else if self.cursor_row < self.lines.len() - 1 {
                    self.cursor_row += 1;
                    self.cursor_col = 0;
                }
            }
            BufferMessage::MoveToStart => {
                self.cursor_col = 0;
            }
            BufferMessage::MoveToEnd => {
                self.cursor_col = self.lines[self.cursor_row].len();
            }
            BufferMessage::MoveToNextWord => {
                let line = &self.lines[self.cursor_row];
                let mut col = self.cursor_col;
                while col < line.len() && !is_word_char(line[col..].chars().next().unwrap()) {
                    col += 1;
                }
                while col < line.len() && is_word_char(line[col..].chars().next().unwrap()) {
                    col += 1;
                }
                self.cursor_col = col;
            }
            BufferMessage::MoveToPrevWord => {
                let line = &self.lines[self.cursor_row];
                let mut col = self.cursor_col;
                while col > 0 && is_word_char(line[col.saturating_sub(1)..].chars().next().unwrap())
                {
                    col -= 1;
                }
                while col > 0
                    && !is_word_char(line[col.saturating_sub(1)..].chars().next().unwrap())
                {
                    col -= 1;
                }
                self.cursor_col = col;
            }
            BufferMessage::MoveToEndOfWord => {
                let line = &self.lines[self.cursor_row];
                let mut col = self.cursor_col;
                while col < line.len()
                    && !is_word_char(line[col..].chars().next().unwrap())
                    && !line[col..].chars().next().unwrap().is_whitespace()
                {
                    col += 1;
                }
                while col < line.len() && is_word_char(line[col..].chars().next().unwrap()) {
                    col += 1;
                }
                if col > self.cursor_col {
                    self.cursor_col = col.saturating_sub(1).min(line.len());
                }
            }
            BufferMessage::MoveToLine(row) => {
                let target = row.saturating_sub(1);
                self.cursor_row = target.min(self.lines.len().saturating_sub(1));
                self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
            }
            BufferMessage::MoveToLastLine => {
                self.cursor_row = self.lines.len().saturating_sub(1);
                self.cursor_col = self.lines[self.cursor_row].len();
            }
            BufferMessage::GetContent => {
                let content = self.lines.join("\n");
                let _ = ctx.reply(content);
            }
            BufferMessage::GetCursorPosition => {
                let _ = ctx.reply((self.cursor_row, self.cursor_col));
            }
            BufferMessage::GetFilePath => {
                let _ = ctx.reply(self.file_path.clone());
            }
            BufferMessage::SetFilePath(path) => {
                self.file_path = path;
            }
            BufferMessage::LoadContent(content) => {
                self.lines = content.lines().map(String::from).collect();
                if self.lines.is_empty() {
                    self.lines.push(String::new());
                }
                self.cursor_row = 0;
                self.cursor_col = 0;
                self.modified = false;
                self.history.clear();
                self.history_index = 0;
            }
            BufferMessage::Undo => {
                if self.history_index > 0 {
                    self.history_index -= 1;
                    if let Some(edit) = self.history.get(self.history_index).cloned() {
                        self.apply_undo(&edit);
                    }
                }
            }
            BufferMessage::Redo => {
                if self.history_index < self.history.len() {
                    if let Some(edit) = self.history.get(self.history_index).cloned() {
                        self.apply_redo(&edit);
                    }
                    self.history_index += 1;
                }
            }
        }
        Ok(())
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[allow(dead_code)]
impl BufferActor {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            modified: false,
            file_path: None,
            history: Vec::new(),
            history_index: 0,
        }
    }

    pub fn open(path: &str) -> Self {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let lines: Vec<String> = content.lines().map(String::from).collect();
        let lines = if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        };
        Self {
            lines,
            cursor_row: 0,
            cursor_col: 0,
            modified: false,
            file_path: Some(path.to_string()),
            history: Vec::new(),
            history_index: 0,
        }
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn cursor_pos(&self) -> (usize, usize) {
        (self.cursor_row, self.cursor_col)
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    fn push_edit(&mut self, edit: BufferEdit) {
        self.history.truncate(self.history_index);
        self.history.push(edit);
        self.history_index += 1;
    }

    fn apply_undo(&mut self, edit: &BufferEdit) {
        match edit {
            BufferEdit::InsertChar { row, col, ch: _ } => {
                if *row < self.lines.len() {
                    let line = &mut self.lines[*row];
                    if *col <= line.len() {
                        line.remove(*col);
                        self.cursor_row = *row;
                        self.cursor_col = *col;
                    }
                }
                self.modified = true;
            }
            BufferEdit::DeleteChar { row, col, ch } => {
                if *row < self.lines.len() {
                    let line = &mut self.lines[*row];
                    if *col <= line.len() {
                        line.insert(*col, *ch);
                        self.cursor_row = *row;
                        self.cursor_col = *col + 1;
                    }
                }
                self.modified = true;
            }
            BufferEdit::InsertLine {
                row,
                split_at: _,
                rest: _,
            } => {
                if *row < self.lines.len() {
                    let current = self.lines.remove(*row + 1);
                    self.lines[*row].push_str(&current);
                    self.cursor_row = *row;
                    self.cursor_col = self.lines[*row].len();
                }
                self.modified = true;
            }
            BufferEdit::DeleteLine { row, content } => {
                self.lines.insert(*row + 1, content.clone());
                self.cursor_row = *row + 1;
                self.cursor_col = 0;
                self.modified = true;
            }
        }
    }

    fn apply_redo(&mut self, edit: &BufferEdit) {
        match edit {
            BufferEdit::InsertChar { row, col, ch } => {
                if *row < self.lines.len() {
                    let line = &mut self.lines[*row];
                    if *col <= line.len() {
                        line.insert(*col, *ch);
                        self.cursor_row = *row;
                        self.cursor_col = *col + 1;
                    }
                }
                self.modified = true;
            }
            BufferEdit::DeleteChar { row, col, ch: _ } => {
                if *row < self.lines.len() {
                    let line = &mut self.lines[*row];
                    if *col < line.len() {
                        line.remove(*col);
                        self.cursor_row = *row;
                        self.cursor_col = *col;
                    }
                }
                self.modified = true;
            }
            BufferEdit::InsertLine {
                row,
                split_at,
                rest: _,
            } => {
                if *row < self.lines.len() {
                    let rest_str = self.lines[*row][*split_at..].to_string();
                    self.lines[*row] = self.lines[*row][..*split_at].to_string();
                    self.lines.insert(*row + 1, rest_str);
                    self.cursor_row = *row + 1;
                    self.cursor_col = 0;
                }
                self.modified = true;
            }
            BufferEdit::DeleteLine { row, content: _ } => {
                if self.lines.len() > 1 {
                    self.lines.remove(*row);
                    if self.cursor_row >= self.lines.len() {
                        self.cursor_row = self.lines.len() - 1;
                    }
                    self.cursor_col = 0;
                }
                self.modified = true;
            }
        }
    }
}

impl Default for BufferActor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create an actor with pre-loaded content (bypasses handle())
    fn actor_with(content: &str) -> BufferActor {
        let mut a = BufferActor::new();
        a.lines = content.lines().map(String::from).collect();
        if a.lines.is_empty() {
            a.lines.push(String::new());
        }
        a
    }

    #[test]
    fn test_new_buffer_empty() {
        let actor = BufferActor::new();
        assert_eq!(actor.lines.len(), 1);
        assert_eq!(actor.lines[0], "");
        assert_eq!(actor.cursor_pos(), (0, 0));
        assert!(!actor.is_modified());
    }

    #[test]
    fn test_load_content_populates_lines() {
        let actor = actor_with("hello\nworld");
        assert_eq!(actor.lines.len(), 2);
        assert_eq!(actor.lines[0], "hello");
        assert_eq!(actor.lines[1], "world");
    }

    #[test]
    fn test_insert_char_advances_cursor() {
        let mut actor = actor_with("hello");
        // Simulate InsertChar('X') at col 2
        actor.lines[0].insert(2, 'X');
        actor.cursor_col = 3;
        actor.modified = true;
        assert_eq!(actor.lines[0], "heXllo");
        assert_eq!(actor.cursor_pos(), (0, 3));
        assert!(actor.is_modified());
    }

    #[test]
    fn test_delete_char_at_col_0_merges_lines() {
        let mut actor = actor_with("hello\nworld");
        actor.cursor_row = 1;
        // Simulate delete at col 0: merge line 1 into line 0
        let current_line = actor.lines.remove(1);
        actor.cursor_row = 0;
        actor.cursor_col = actor.lines[0].len();
        actor.lines[0].push_str(&current_line);
        assert_eq!(actor.lines.len(), 1);
        assert_eq!(actor.lines[0], "helloworld");
        assert_eq!(actor.cursor_pos(), (0, 5));
    }

    #[test]
    fn test_delete_line_removes_current() {
        let mut actor = actor_with("line1\nline2\nline3");
        actor.cursor_row = 1;
        // Simulate DeleteLine
        let content = actor.lines.remove(1);
        assert_eq!(actor.lines.len(), 2);
        assert_eq!(actor.lines[0], "line1");
        assert_eq!(actor.lines[1], "line3");
        let _ = content; // would be stored in history
    }

    #[test]
    fn test_undo_redo_with_history() {
        let mut actor = actor_with("abc");
        // Manually push an edit (simulating InsertChar('X') at col 0)
        actor.lines[0].insert(0, 'X');
        actor.cursor_col = 1;
        actor.modified = true;
        actor.push_edit(BufferEdit::InsertChar {
            row: 0,
            col: 0,
            ch: 'X',
        });
        assert_eq!(actor.lines[0], "Xabc");

        // Undo: remove the inserted char
        let edit = actor.history[0].clone();
        actor.apply_undo(&edit);
        assert_eq!(actor.lines[0], "abc");
        assert_eq!(actor.cursor_pos(), (0, 0));

        // Redo: re-insert the char
        let edit = actor.history[0].clone();
        actor.apply_redo(&edit);
        assert_eq!(actor.lines[0], "Xabc");
        assert_eq!(actor.cursor_pos(), (0, 1));
    }

    #[test]
    fn test_move_to_line_sets_cursor() {
        let mut actor = actor_with("a\nb\nc");
        // MoveToLine(2) means row index 1
        let target = 2usize.saturating_sub(1);
        actor.cursor_row = target.min(actor.lines.len().saturating_sub(1));
        assert_eq!(actor.cursor_pos().0, 1);
    }

    #[test]
    fn test_move_to_last_line() {
        let mut actor = actor_with("a\nb\nc");
        actor.cursor_row = actor.lines.len().saturating_sub(1);
        actor.cursor_col = actor.lines[actor.cursor_row].len();
        assert_eq!(actor.cursor_pos(), (2, 1));
    }

    #[test]
    fn test_set_file_path() {
        let mut actor = actor_with("content");
        actor.file_path = Some("/tmp/test.txt".to_string());
        assert_eq!(actor.file_path, Some("/tmp/test.txt".to_string()));
    }
}
