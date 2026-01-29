use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::PathBuf;

use super::types::ContextState;

/// Trait for context storage (enables mocking in tests)
pub trait ContextStore {
    fn load(&self) -> Result<ContextState>;
    fn save(&self, state: &ContextState) -> Result<()>;
    fn delete(&self) -> Result<()>;
}

/// File-based context store
pub struct FileContextStore {
    path: PathBuf,
    session_id: String,
}

impl FileContextStore {
    /// Create a new file context store
    pub fn new() -> Result<Self> {
        let session_id = get_session_id();
        let path = context_file_path(&session_id);
        Ok(Self { path, session_id })
    }

    /// Create with explicit path (for testing)
    #[cfg(test)]
    pub fn with_path(path: PathBuf, session_id: String) -> Self {
        Self { path, session_id }
    }
}

impl ContextStore for FileContextStore {
    fn load(&self) -> Result<ContextState> {
        if !self.path.exists() {
            return Ok(ContextState::new(self.session_id.clone()));
        }

        let contents = fs::read_to_string(&self.path)
            .with_context(|| format!("Failed to read {}", self.path.display()))?;

        serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse {}", self.path.display()))
    }

    fn save(&self, state: &ContextState) -> Result<()> {
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir)
                .with_context(|| format!("Failed to create directory {}", dir.display()))?;
        }

        let contents =
            serde_json::to_string_pretty(state).context("Failed to serialize context state")?;

        fs::write(&self.path, contents)
            .with_context(|| format!("Failed to write {}", self.path.display()))?;

        Ok(())
    }

    fn delete(&self) -> Result<()> {
        if self.path.exists() {
            fs::remove_file(&self.path)
                .with_context(|| format!("Failed to delete {}", self.path.display()))?;
        }
        Ok(())
    }
}

/// Get session ID from environment or generate one
fn get_session_id() -> String {
    env::var("CLAUDE_SESSION_ID").unwrap_or_else(|_| {
        // Use a stable ID based on process ID and start time
        // This gives same ID within a session but different across sessions
        format!("hu-{}", std::process::id())
    })
}

/// Get path to context file for a session
fn context_file_path(session_id: &str) -> PathBuf {
    let tmp = env::temp_dir();
    tmp.join(format!("hu-context-{}.json", session_id))
}

/// Get the default store instance
pub fn default_store() -> Result<FileContextStore> {
    FileContextStore::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::types::ContextEntry;

    fn temp_store() -> (FileContextStore, PathBuf) {
        let tmp_dir = std::env::temp_dir().join(format!("hu_store_test_{}", rand_suffix()));
        let _ = std::fs::create_dir_all(&tmp_dir);
        let path = tmp_dir.join("context.json");
        let store = FileContextStore::with_path(path.clone(), "test-session".to_string());
        (store, tmp_dir)
    }

    fn rand_suffix() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }

    #[test]
    fn file_context_store_new() {
        let store = FileContextStore::new().unwrap();
        assert!(!store.session_id.is_empty());
    }

    #[test]
    fn file_context_store_with_path() {
        let store = FileContextStore::with_path(PathBuf::from("/test/path"), "sid".to_string());
        assert_eq!(store.path, PathBuf::from("/test/path"));
        assert_eq!(store.session_id, "sid");
    }

    #[test]
    fn store_session_id() {
        let store = FileContextStore::with_path(PathBuf::from("/tmp/x"), "my-session".to_string());
        assert_eq!(store.session_id, "my-session");
    }

    #[test]
    fn store_load_missing_file() {
        let (store, tmp_dir) = temp_store();
        let state = store.load().unwrap();
        assert_eq!(state.session_id, "test-session");
        assert!(state.entries.is_empty());
        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn store_save_and_load() {
        let (store, tmp_dir) = temp_store();

        let mut state = ContextState::new("test-session".to_string());
        state.track(ContextEntry::with_timestamp(
            PathBuf::from("/test.rs"),
            100,
            10,
            123,
        ));

        store.save(&state).unwrap();
        assert!(store.path.exists());

        let loaded = store.load().unwrap();
        assert_eq!(loaded.session_id, "test-session");
        assert_eq!(loaded.file_count(), 1);

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn store_delete() {
        let (store, tmp_dir) = temp_store();

        let state = ContextState::new("test-session".to_string());
        store.save(&state).unwrap();
        assert!(store.path.exists());

        store.delete().unwrap();
        assert!(!store.path.exists());

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn store_delete_missing_file() {
        let (store, tmp_dir) = temp_store();
        // Delete should succeed even if file doesn't exist
        store.delete().unwrap();
        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn store_save_creates_parent_dirs() {
        let tmp_dir = std::env::temp_dir().join(format!("hu_nested_{}/a/b", rand_suffix()));
        let path = tmp_dir.join("context.json");
        let store = FileContextStore::with_path(path.clone(), "s".to_string());

        let state = ContextState::new("s".to_string());
        store.save(&state).unwrap();
        assert!(path.exists());

        let _ = std::fs::remove_dir_all(
            std::env::temp_dir().join(format!("hu_nested_{}", rand_suffix())),
        );
    }

    #[test]
    fn store_overwrites_existing() {
        let (store, tmp_dir) = temp_store();

        let mut state1 = ContextState::new("test-session".to_string());
        state1.track(ContextEntry::with_timestamp(
            PathBuf::from("/old.rs"),
            1,
            1,
            1,
        ));
        store.save(&state1).unwrap();

        let mut state2 = ContextState::new("test-session".to_string());
        state2.track(ContextEntry::with_timestamp(
            PathBuf::from("/new.rs"),
            2,
            2,
            2,
        ));
        store.save(&state2).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.file_count(), 1);
        assert!(loaded.is_tracked(&PathBuf::from("/new.rs")));
        assert!(!loaded.is_tracked(&PathBuf::from("/old.rs")));

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn get_session_id_from_env() {
        // This test just verifies the function runs
        let id = get_session_id();
        assert!(!id.is_empty());
    }

    #[test]
    fn context_file_path_format() {
        let path = context_file_path("my-session-123");
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("hu-context-my-session-123.json"));
    }

    #[test]
    fn default_store_creates() {
        let store = default_store().unwrap();
        assert!(!store.session_id.is_empty());
    }
}
