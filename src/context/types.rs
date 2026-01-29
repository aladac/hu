use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Entry for a tracked file in the context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextEntry {
    /// Absolute path to the file
    pub path: PathBuf,
    /// File size in bytes
    pub size: u64,
    /// Number of lines in the file
    pub line_count: usize,
    /// When the file was tracked (Unix timestamp)
    pub tracked_at: u64,
}

impl ContextEntry {
    /// Create a new context entry from file metadata
    pub fn new(path: PathBuf, size: u64, line_count: usize) -> Self {
        let tracked_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            path,
            size,
            line_count,
            tracked_at,
        }
    }

    /// Create entry with explicit timestamp (for testing)
    #[cfg(test)]
    pub fn with_timestamp(path: PathBuf, size: u64, line_count: usize, tracked_at: u64) -> Self {
        Self {
            path,
            size,
            line_count,
            tracked_at,
        }
    }
}

/// Complete context state for a session
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ContextState {
    /// Session ID this context belongs to
    pub session_id: String,
    /// Map of canonical path string to entry
    pub entries: HashMap<String, ContextEntry>,
}

impl ContextState {
    /// Create a new empty context state
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            entries: HashMap::new(),
        }
    }

    /// Add or update an entry
    pub fn track(&mut self, entry: ContextEntry) {
        let key = entry.path.to_string_lossy().to_string();
        self.entries.insert(key, entry);
    }

    /// Get an entry by path
    pub fn get(&self, path: &Path) -> Option<&ContextEntry> {
        let key = path.to_string_lossy().to_string();
        self.entries.get(&key)
    }

    /// Check if a path is tracked
    #[cfg(test)]
    pub fn is_tracked(&self, path: &Path) -> bool {
        let key = path.to_string_lossy().to_string();
        self.entries.contains_key(&key)
    }

    /// Get all entries
    pub fn all_entries(&self) -> Vec<&ContextEntry> {
        self.entries.values().collect()
    }

    /// Clear all entries
    #[cfg(test)]
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Total tracked files
    pub fn file_count(&self) -> usize {
        self.entries.len()
    }

    /// Total bytes tracked
    pub fn total_bytes(&self) -> u64 {
        self.entries.values().map(|e| e.size).sum()
    }

    /// Total lines tracked
    pub fn total_lines(&self) -> usize {
        self.entries.values().map(|e| e.line_count).sum()
    }
}

/// Result of checking a file's status
#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    /// File is already in context
    Loaded { entry: ContextEntry, age_secs: u64 },
    /// File is not in context
    NotLoaded {
        path: PathBuf,
        size: u64,
        line_count: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_entry_new() {
        let entry = ContextEntry::new(PathBuf::from("/test/file.rs"), 1024, 50);
        assert_eq!(entry.path, PathBuf::from("/test/file.rs"));
        assert_eq!(entry.size, 1024);
        assert_eq!(entry.line_count, 50);
        assert!(entry.tracked_at > 0);
    }

    #[test]
    fn context_entry_with_timestamp() {
        let entry = ContextEntry::with_timestamp(PathBuf::from("/test.rs"), 512, 25, 1000);
        assert_eq!(entry.tracked_at, 1000);
    }

    #[test]
    fn context_entry_clone() {
        let entry = ContextEntry::new(PathBuf::from("/test.rs"), 100, 10);
        let cloned = entry.clone();
        assert_eq!(entry, cloned);
    }

    #[test]
    fn context_entry_debug() {
        let entry = ContextEntry::new(PathBuf::from("/test.rs"), 100, 10);
        let debug = format!("{:?}", entry);
        assert!(debug.contains("ContextEntry"));
    }

    #[test]
    fn context_entry_serialize_deserialize() {
        let entry = ContextEntry::with_timestamp(PathBuf::from("/test.rs"), 100, 10, 12345);
        let json = serde_json::to_string(&entry).unwrap();
        let parsed: ContextEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, parsed);
    }

    #[test]
    fn context_state_new() {
        let state = ContextState::new("session123".to_string());
        assert_eq!(state.session_id, "session123");
        assert!(state.entries.is_empty());
    }

    #[test]
    fn context_state_default() {
        let state = ContextState::default();
        assert_eq!(state.session_id, "");
        assert!(state.entries.is_empty());
    }

    #[test]
    fn context_state_track() {
        let mut state = ContextState::new("s1".to_string());
        let entry = ContextEntry::new(PathBuf::from("/a.rs"), 100, 10);
        state.track(entry.clone());

        assert_eq!(state.file_count(), 1);
        assert!(state.is_tracked(&PathBuf::from("/a.rs")));
    }

    #[test]
    fn context_state_get() {
        let mut state = ContextState::new("s1".to_string());
        let entry = ContextEntry::with_timestamp(PathBuf::from("/a.rs"), 100, 10, 500);
        state.track(entry);

        let got = state.get(&PathBuf::from("/a.rs")).unwrap();
        assert_eq!(got.size, 100);
        assert_eq!(got.line_count, 10);
    }

    #[test]
    fn context_state_get_missing() {
        let state = ContextState::new("s1".to_string());
        assert!(state.get(&PathBuf::from("/missing.rs")).is_none());
    }

    #[test]
    fn context_state_is_tracked() {
        let mut state = ContextState::new("s1".to_string());
        state.track(ContextEntry::new(PathBuf::from("/yes.rs"), 1, 1));

        assert!(state.is_tracked(&PathBuf::from("/yes.rs")));
        assert!(!state.is_tracked(&PathBuf::from("/no.rs")));
    }

    #[test]
    fn context_state_all_entries() {
        let mut state = ContextState::new("s1".to_string());
        state.track(ContextEntry::new(PathBuf::from("/a.rs"), 100, 10));
        state.track(ContextEntry::new(PathBuf::from("/b.rs"), 200, 20));

        let entries = state.all_entries();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn context_state_clear() {
        let mut state = ContextState::new("s1".to_string());
        state.track(ContextEntry::new(PathBuf::from("/a.rs"), 100, 10));
        assert_eq!(state.file_count(), 1);

        state.clear();
        assert_eq!(state.file_count(), 0);
    }

    #[test]
    fn context_state_totals() {
        let mut state = ContextState::new("s1".to_string());
        state.track(ContextEntry::new(PathBuf::from("/a.rs"), 100, 10));
        state.track(ContextEntry::new(PathBuf::from("/b.rs"), 200, 20));

        assert_eq!(state.total_bytes(), 300);
        assert_eq!(state.total_lines(), 30);
    }

    #[test]
    fn context_state_update_existing() {
        let mut state = ContextState::new("s1".to_string());
        state.track(ContextEntry::with_timestamp(
            PathBuf::from("/a.rs"),
            100,
            10,
            1,
        ));
        state.track(ContextEntry::with_timestamp(
            PathBuf::from("/a.rs"),
            200,
            20,
            2,
        ));

        assert_eq!(state.file_count(), 1);
        let entry = state.get(&PathBuf::from("/a.rs")).unwrap();
        assert_eq!(entry.size, 200);
        assert_eq!(entry.tracked_at, 2);
    }

    #[test]
    fn context_state_serialize_deserialize() {
        let mut state = ContextState::new("session".to_string());
        state.track(ContextEntry::with_timestamp(
            PathBuf::from("/test.rs"),
            100,
            10,
            123,
        ));

        let json = serde_json::to_string(&state).unwrap();
        let parsed: ContextState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, parsed);
    }

    #[test]
    fn context_state_clone() {
        let mut state = ContextState::new("s1".to_string());
        state.track(ContextEntry::new(PathBuf::from("/a.rs"), 100, 10));

        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn context_state_debug() {
        let state = ContextState::new("s1".to_string());
        let debug = format!("{:?}", state);
        assert!(debug.contains("ContextState"));
    }

    #[test]
    fn file_status_loaded() {
        let entry = ContextEntry::with_timestamp(PathBuf::from("/a.rs"), 100, 10, 1);
        let status = FileStatus::Loaded {
            entry: entry.clone(),
            age_secs: 60,
        };
        if let FileStatus::Loaded { entry: e, age_secs } = status {
            assert_eq!(e.path, PathBuf::from("/a.rs"));
            assert_eq!(age_secs, 60);
        } else {
            panic!("Expected Loaded");
        }
    }

    #[test]
    fn file_status_not_loaded() {
        let status = FileStatus::NotLoaded {
            path: PathBuf::from("/b.rs"),
            size: 500,
            line_count: 50,
        };
        if let FileStatus::NotLoaded {
            path,
            size,
            line_count,
        } = status
        {
            assert_eq!(path, PathBuf::from("/b.rs"));
            assert_eq!(size, 500);
            assert_eq!(line_count, 50);
        } else {
            panic!("Expected NotLoaded");
        }
    }

    #[test]
    fn file_status_clone() {
        let status = FileStatus::NotLoaded {
            path: PathBuf::from("/a.rs"),
            size: 100,
            line_count: 10,
        };
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }

    #[test]
    fn file_status_debug() {
        let status = FileStatus::NotLoaded {
            path: PathBuf::from("/a.rs"),
            size: 100,
            line_count: 10,
        };
        let debug = format!("{:?}", status);
        assert!(debug.contains("NotLoaded"));
    }
}
