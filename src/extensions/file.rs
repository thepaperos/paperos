use crate::core::{BufferMessage, CommandHandler};
use runact::{ActorId, Runtime};
use std::any::Any;
use std::sync::{Arc, Mutex};

#[allow(dead_code)]
pub struct FileExtension {
    buffer: Option<ActorId>,
    file_path: Option<String>,
}

impl CommandHandler for FileExtension {
    fn handle(
        &mut self,
        args: &[String],
        runtime: &Runtime,
        buffer: Option<ActorId>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let buffer_id = buffer.or(self.buffer).expect("no buffer set");

        match args.get(0).map(|s| s.as_str()) {
            Some("open") | Some("e") | Some("edit") => {
                if args.len() < 2 {
                    println!("Usage: open <file>");
                    return Ok(());
                }
                let path = expand_tilde(&args[1]);
                let content = std::fs::read_to_string(&path)?;
                self.file_path = Some(path.clone());
                let _ = runtime.send(buffer_id, BufferMessage::SetFilePath(Some(path.clone())));
                let _ = runtime.send(buffer_id, BufferMessage::LoadContent(content));
            }
            Some("write") | Some("w") | Some("save") => {
                let path = args
                    .get(1)
                    .map(|s| expand_tilde(s))
                    .or_else(|| self.file_path.clone())
                    .unwrap_or_else(|| "unnamed".to_string());
                if let Ok(handle) = runtime.request(buffer_id, BufferMessage::GetContent) {
                    if let Ok(reply) = handle.recv_timeout(std::time::Duration::from_millis(100)) {
                        if let Ok(content) = reply.downcast::<String>() {
                            std::fs::write(&path, &*content)?;
                            self.file_path = Some(path.clone());
                            let _ = runtime.send(buffer_id, BufferMessage::SetFilePath(Some(path)));
                        }
                    }
                }
            }
            _ => println!("Unknown file command: {:?}", args.get(0)),
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

#[allow(dead_code)]
impl FileExtension {
    pub fn new() -> Self {
        Self {
            buffer: None,
            file_path: None,
        }
    }

    pub fn set_buffer(&mut self, buffer: ActorId) {
        self.buffer = Some(buffer);
    }

    pub fn buffer(&self) -> Option<ActorId> {
        self.buffer
    }

    pub fn file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }
}

impl Default for FileExtension {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared wrapper so multiple command names can dispatch to the same handler.
#[allow(dead_code)]
pub struct SharedFileCommands {
    inner: Arc<Mutex<FileExtension>>,
}

impl SharedFileCommands {
    pub fn new(ext: Arc<Mutex<FileExtension>>) -> Self {
        Self { inner: ext }
    }

    pub fn clone_handler(&self) -> SharedFileCommands {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl CommandHandler for SharedFileCommands {
    fn handle(
        &mut self,
        args: &[String],
        runtime: &Runtime,
        buffer: Option<ActorId>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut ext = self.inner.lock().unwrap();
        ext.handle(args, runtime, buffer)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{}{}", home, &path[1..]);
        }
    }
    path.to_string()
}
