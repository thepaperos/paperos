use crate::core::{BufferMessage, CommandHandler};
use runact::{ActorId, Runtime};
use std::any::Any;
use std::sync::{Arc, Mutex};

/// Command handler that maps editor commands to BufferMessage sends.
/// Every buffer operation (movement, insertion, deletion, undo/redo) is
/// registered as a command so users can invoke them via `:command`.
#[allow(dead_code)]
pub struct BufferCommands {
    buffer: Option<ActorId>,
}

impl CommandHandler for BufferCommands {
    fn handle(
        &mut self,
        args: &[String],
        runtime: &Runtime,
        buffer: Option<ActorId>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let buffer_id = buffer.or(self.buffer).expect("no buffer set");

        let cmd = args.get(0).map(|s| s.as_str()).unwrap_or("");

        let msg = match cmd {
            "left" | "h" => Some(BufferMessage::MoveLeft),
            "right" | "l" => Some(BufferMessage::MoveRight),
            "up" | "k" => Some(BufferMessage::MoveUp),
            "down" | "j" => Some(BufferMessage::MoveDown),
            "home" | "^" | "0" => Some(BufferMessage::MoveToStart),
            "end" | "$" => Some(BufferMessage::MoveToEnd),
            "w" | "nextword" => Some(BufferMessage::MoveToNextWord),
            "b" | "prevword" => Some(BufferMessage::MoveToPrevWord),
            "e" | "endword" => Some(BufferMessage::MoveToEndOfWord),
            "go" | "line" => {
                if let Some(n) = args.get(1).and_then(|s| s.parse::<usize>().ok()) {
                    Some(BufferMessage::MoveToLine(n))
                } else {
                    None
                }
            }
            "lastline" | "G" => Some(BufferMessage::MoveToLastLine),
            "insert" | "i" => {
                if let Some(arg) = args.get(1) {
                    if let Some(ch) = arg.chars().next() {
                        Some(BufferMessage::InsertChar(ch))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "delete" | "x" => Some(BufferMessage::DeleteChar),
            "deleteline" | "dd" => Some(BufferMessage::DeleteLine),
            "newline" | "enter" => Some(BufferMessage::InsertLine),
            "undo" | "u" => Some(BufferMessage::Undo),
            "redo" | "U" | "ctrl-r" => Some(BufferMessage::Redo),
            _ => None,
        };

        if let Some(message) = msg {
            let _ = runtime.send(buffer_id, message);
        } else {
            println!("Unknown command: {}", cmd);
        }

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl BufferCommands {
    pub fn new() -> Self {
        Self { buffer: None }
    }

    pub fn set_buffer(&mut self, buffer: ActorId) {
        self.buffer = Some(buffer);
    }
}

impl Default for BufferCommands {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared wrapper so multiple command names can dispatch to the same handler.
#[allow(dead_code)]
pub struct SharedBufferCommands {
    inner: Arc<Mutex<BufferCommands>>,
}

impl SharedBufferCommands {
    pub fn new(cmds: BufferCommands) -> Self {
        Self {
            inner: Arc::new(Mutex::new(cmds)),
        }
    }

    pub fn clone_handler(&self) -> SharedBufferCommands {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl CommandHandler for SharedBufferCommands {
    fn handle(
        &mut self,
        args: &[String],
        runtime: &Runtime,
        buffer: Option<ActorId>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut cmds = self.inner.lock().unwrap();
        cmds.handle(args, runtime, buffer)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
