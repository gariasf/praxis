//! Hot-reload support for script files.

use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher,
};
use parking_lot::Mutex;
use praxis_utils::{debug, error, info, Result};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;

/// Event types for script changes.
#[derive(Debug, Clone)]
pub enum ScriptEvent {
    /// Script file was modified
    Modified(PathBuf),

    /// Script file was removed
    Removed(PathBuf),
}

/// Watches script files for changes and enables hot-reload.
pub struct HotReloadWatcher {
    _watcher: RecommendedWatcher,
    receiver: Arc<Mutex<Receiver<NotifyResult<Event>>>>,
    events: Vec<ScriptEvent>,
}

impl HotReloadWatcher {
    /// Creates a new hot-reload watcher for the given directory.
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        info!("Setting up hot-reload watcher for {:?}", path);

        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                if tx.send(res).is_err() {
                    error!("Failed to send file system event");
                }
            },
            Config::default(),
        )
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create watcher: {}", e))?;

        watcher
            .watch(path, RecursiveMode::Recursive)
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to watch path: {}", e))?;

        Ok(Self {
            _watcher: watcher,
            receiver: Arc::new(Mutex::new(rx)),
            events: Vec::new(),
        })
    }

    /// Polls for new file system events and returns any script events.
    pub fn poll_events(&mut self) -> Vec<ScriptEvent> {
        let events_to_process = {
            let receiver = self.receiver.lock();
            let mut temp_events = Vec::new();

            while let Ok(result) = receiver.try_recv() {
                temp_events.push(result);
            }
            temp_events
        };

        for result in events_to_process {
            match result {
                Ok(event) => {
                    self.process_event(event);
                }
                Err(e) => {
                    error!("File system event error: {}", e);
                }
            }
        }

        std::mem::take(&mut self.events)
    }

    fn process_event(&mut self, event: Event) {
        match event.kind {
            EventKind::Modify(_) => {
                for path in event.paths {
                    if is_script_file(&path) {
                        debug!("Script modified: {:?}", path);
                        self.events.push(ScriptEvent::Modified(path));
                    }
                }
            }
            EventKind::Remove(_) => {
                for path in event.paths {
                    if is_script_file(&path) {
                        debug!("Script removed: {:?}", path);
                        self.events.push(ScriptEvent::Removed(path));
                    }
                }
            }
            _ => {}
        }
    }
}

fn is_script_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "lua")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_create_watcher() {
        let temp_dir = TempDir::new().unwrap();
        let watcher = HotReloadWatcher::new(temp_dir.path());
        assert!(watcher.is_ok());
    }

    #[test]
    fn test_detect_file_modification() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("test.lua");

        fs::write(&script_path, "-- test").unwrap();

        let mut watcher = HotReloadWatcher::new(temp_dir.path()).unwrap();

        thread::sleep(Duration::from_millis(100));

        fs::write(&script_path, "-- modified").unwrap();

        thread::sleep(Duration::from_millis(200));

        let events = watcher.poll_events();
        assert!(!events.is_empty());

        let has_modified = events.iter().any(|e| matches!(e, ScriptEvent::Modified(_)));
        assert!(has_modified);
    }

    #[test]
    fn test_ignore_non_lua_files() {
        let temp_dir = TempDir::new().unwrap();
        let txt_path = temp_dir.path().join("test.txt");

        fs::write(&txt_path, "test").unwrap();

        let mut watcher = HotReloadWatcher::new(temp_dir.path()).unwrap();

        thread::sleep(Duration::from_millis(100));

        fs::write(&txt_path, "modified").unwrap();

        thread::sleep(Duration::from_millis(200));

        let events = watcher.poll_events();
        assert!(events.is_empty());
    }
}
