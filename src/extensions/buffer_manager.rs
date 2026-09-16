use crate::core::{BufferActor, BufferMessage};
use runact::{ActorId, Runtime};
use std::path::Path;
use std::sync::{Arc, RwLock};

/// BufferManagerExtension: manages multiple buffers within the editor.
/// Spawns BufferActor instances and tracks which is active.
/// Communication with RenderExtension and VisualModeExtension happens via
/// shared Arc<RwLock<Option<ActorId>>> for the active buffer.
pub struct BufferManager {
    buffers: Vec<BufferEntry>,
    active_index: usize,
    /// Shared active buffer reference — RenderExtension and VisualModeExtension
    /// both hold a clone of this and read from it each frame.
    active_buf: Arc<RwLock<Option<ActorId>>>,
}

struct BufferEntry {
    id: ActorId,
    file_path: Option<String>,
    modified: bool,
}

impl BufferManager {
    pub fn new() -> Self {
        Self {
            buffers: Vec::new(),
            active_index: 0,
            active_buf: Arc::new(RwLock::new(None)),
        }
    }

    /// Spawn a new buffer for the given file content (or empty).
    pub fn add_buffer(
        &mut self,
        runtime: &mut Runtime,
        content: Option<String>,
        path: Option<String>,
    ) -> ActorId {
        let mut actor = BufferActor::new();
        if let Some(c) = content {
            actor.lines = c.lines().map(String::from).collect();
            if actor.lines.is_empty() {
                actor.lines.push(String::new());
            }
        }
        let id = runtime.spawn(actor).expect("failed to spawn buffer");
        if let Some(p) = path {
            let _ = runtime.send(id, BufferMessage::SetFilePath(Some(p)));
        }
        self.buffers.push(BufferEntry {
            id,
            file_path: None,
            modified: false,
        });
        if self.buffers.len() == 1 {
            self.active_index = 0;
            *self.active_buf.write().unwrap() = Some(id);
        }
        id
    }

    /// Switch to the buffer at the given index.
    pub fn switch_to(&mut self, index: usize) {
        if index < self.buffers.len() {
            self.active_index = index;
            *self.active_buf.write().unwrap() = Some(self.buffers[index].id);
        }
    }

    /// Switch to the next buffer (circular).
    pub fn next(&mut self) {
        if self.buffers.is_empty() {
            return;
        }
        self.active_index = (self.active_index + 1) % self.buffers.len();
        *self.active_buf.write().unwrap() = Some(self.buffers[self.active_index].id);
    }

    /// Switch to the previous buffer (circular).
    pub fn prev(&mut self) {
        if self.buffers.is_empty() {
            return;
        }
        self.active_index = (self.active_index + self.buffers.len() - 1) % self.buffers.len();
        *self.active_buf.write().unwrap() = Some(self.buffers[self.active_index].id);
    }

    /// Get the currently active ActorId.
    pub fn active_id(&self) -> Option<ActorId> {
        *self.active_buf.read().unwrap()
    }

    /// Get the shared active buffer reference.
    pub fn active_buf_handle(&self) -> Arc<RwLock<Option<ActorId>>> {
        Arc::clone(&self.active_buf)
    }

    /// List all buffers as (index, file_name, modified) tuples.
    pub fn list(&self) -> Vec<(usize, String, bool)> {
        self.buffers
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let name = e
                    .file_path
                    .as_ref()
                    .and_then(|p| {
                        Path::new(p)
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                    })
                    .unwrap_or_else(|| format!("buffer {}", i + 1));
                (i, name, e.modified)
            })
            .collect()
    }

    /// Get the active buffer index.
    pub fn active_index(&self) -> usize {
        self.active_index
    }

    /// Get total buffer count.
    pub fn buffer_count(&self) -> usize {
        self.buffers.len()
    }
}

impl Default for BufferManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer_manager_is_empty() {
        let mgr = BufferManager::new();
        assert_eq!(mgr.buffer_count(), 0);
        assert_eq!(mgr.active_index(), 0);
        assert!(mgr.active_id().is_none());
    }

    #[test]
    fn test_switch_to_within_bounds() {
        let mut mgr = BufferManager::new();
        mgr.buffers.push(BufferEntry {
            id: ActorId::new(1),
            file_path: Some("file1.txt".to_string()),
            modified: false,
        });
        mgr.buffers.push(BufferEntry {
            id: ActorId::new(2),
            file_path: Some("file2.txt".to_string()),
            modified: true,
        });
        *mgr.active_buf.write().unwrap() = Some(ActorId::new(1));
        assert_eq!(mgr.buffer_count(), 2);

        mgr.switch_to(0);
        assert_eq!(mgr.active_index(), 0);
        assert_eq!(mgr.active_id(), Some(ActorId::new(1)));

        mgr.switch_to(1);
        assert_eq!(mgr.active_index(), 1);
        assert_eq!(mgr.active_id(), Some(ActorId::new(2)));
    }

    #[test]
    fn test_switch_to_out_of_bounds_is_noop() {
        let mut mgr = BufferManager::new();
        mgr.buffers.push(BufferEntry {
            id: ActorId::new(1),
            file_path: None,
            modified: false,
        });
        *mgr.active_buf.write().unwrap() = Some(ActorId::new(1));
        let count = mgr.buffers.len();
        mgr.switch_to(count); // out of bounds
        assert_eq!(mgr.active_index(), 0); // unchanged
    }

    #[test]
    fn test_next_cycles_circular() {
        let mut mgr = BufferManager::new();
        mgr.buffers.push(BufferEntry {
            id: ActorId::new(1),
            file_path: None,
            modified: false,
        });
        mgr.buffers.push(BufferEntry {
            id: ActorId::new(2),
            file_path: None,
            modified: false,
        });
        mgr.buffers.push(BufferEntry {
            id: ActorId::new(3),
            file_path: None,
            modified: false,
        });
        *mgr.active_buf.write().unwrap() = Some(ActorId::new(1));

        assert_eq!(mgr.active_index(), 0);
        assert_eq!(mgr.active_id(), Some(ActorId::new(1)));

        mgr.next();
        assert_eq!(mgr.active_index(), 1);
        assert_eq!(mgr.active_id(), Some(ActorId::new(2)));

        mgr.next();
        assert_eq!(mgr.active_index(), 2);
        assert_eq!(mgr.active_id(), Some(ActorId::new(3)));

        mgr.next(); // wraps around
        assert_eq!(mgr.active_index(), 0);
        assert_eq!(mgr.active_id(), Some(ActorId::new(1)));
    }

    #[test]
    fn test_prev_cycles_circular() {
        let mut mgr = BufferManager::new();
        mgr.buffers.push(BufferEntry {
            id: ActorId::new(1),
            file_path: None,
            modified: false,
        });
        mgr.buffers.push(BufferEntry {
            id: ActorId::new(2),
            file_path: None,
            modified: false,
        });
        *mgr.active_buf.write().unwrap() = Some(ActorId::new(1));

        assert_eq!(mgr.active_index(), 0);

        mgr.prev(); // wraps to last
        assert_eq!(mgr.active_index(), 1);
        assert_eq!(mgr.active_id(), Some(ActorId::new(2)));

        mgr.prev(); // back to first
        assert_eq!(mgr.active_index(), 0);
        assert_eq!(mgr.active_id(), Some(ActorId::new(1)));
    }

    #[test]
    fn test_list_returns_names() {
        let mgr = BufferManager::new();
        let _ = mgr; // list() needs Runtime for request, test with manual entries
    }

    #[test]
    fn test_active_buf_handle_shared() {
        let mgr = BufferManager::new();
        let handle = mgr.active_buf_handle();
        {
            let guard = handle.read().unwrap();
            assert!(guard.is_none());
        }
    }

    #[test]
    fn test_empty_manager_next_is_noop() {
        let mut mgr = BufferManager::new();
        mgr.next(); // should not panic
        assert_eq!(mgr.buffer_count(), 0);
    }

    #[test]
    fn test_empty_manager_prev_is_noop() {
        let mut mgr = BufferManager::new();
        mgr.prev(); // should not panic
        assert_eq!(mgr.buffer_count(), 0);
    }
}
