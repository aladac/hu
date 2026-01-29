use anyhow::{Context, Result};
use std::fs;
use std::io::BufRead;
use std::path::PathBuf;
use std::time::SystemTime;

use super::store::{default_store, ContextStore};
use super::types::{ContextEntry, ContextState, FileStatus};

/// Track file(s) as loaded in context
pub async fn track(paths: &[String]) -> Result<()> {
    let store = default_store()?;
    track_with_store(&store, paths)
}

/// Track files using a specific store (for testing)
pub fn track_with_store(store: &impl ContextStore, paths: &[String]) -> Result<()> {
    let mut state = store.load()?;

    for path_str in paths {
        let path = resolve_path(path_str)?;
        let (size, line_count) = get_file_info(&path)?;
        let entry = ContextEntry::new(path.clone(), size, line_count);
        state.track(entry);
        println!(
            "Tracked: {} ({} lines, {} bytes)",
            path.display(),
            line_count,
            size
        );
    }

    store.save(&state)?;
    Ok(())
}

/// Check if file(s) are in context
pub async fn check(paths: &[String]) -> Result<()> {
    let store = default_store()?;
    check_with_store(&store, paths)
}

/// Check files using a specific store (for testing)
pub fn check_with_store(store: &impl ContextStore, paths: &[String]) -> Result<()> {
    let state = store.load()?;
    let now = current_timestamp();

    for path_str in paths {
        let path = resolve_path(path_str)?;
        let status = get_file_status(&state, &path, now)?;
        print_file_status(&status);
    }

    Ok(())
}

/// Show summary of all tracked files
pub async fn summary() -> Result<()> {
    let store = default_store()?;
    summary_with_store(&store)
}

/// Show summary using a specific store (for testing)
pub fn summary_with_store(store: &impl ContextStore) -> Result<()> {
    let state = store.load()?;
    let now = current_timestamp();

    if state.file_count() == 0 {
        println!("No files tracked in context");
        return Ok(());
    }

    println!("Session: {}", state.session_id);
    println!();

    let mut entries: Vec<_> = state.all_entries().into_iter().collect();
    entries.sort_by(|a, b| b.tracked_at.cmp(&a.tracked_at));

    for entry in &entries {
        let age = format_age(now.saturating_sub(entry.tracked_at));
        println!(
            "  {} ({} lines, {}) - {}",
            entry.path.display(),
            entry.line_count,
            format_bytes(entry.size),
            age
        );
    }

    println!();
    println!(
        "Total: {} files, {} lines, {}",
        state.file_count(),
        state.total_lines(),
        format_bytes(state.total_bytes())
    );

    Ok(())
}

/// Clear all tracked files
pub async fn clear() -> Result<()> {
    let store = default_store()?;
    clear_with_store(&store)
}

/// Clear using a specific store (for testing)
pub fn clear_with_store(store: &impl ContextStore) -> Result<()> {
    store.delete()?;
    println!("Context cleared");
    Ok(())
}

/// Get file status relative to current context
pub fn get_file_status(state: &ContextState, path: &PathBuf, now: u64) -> Result<FileStatus> {
    if let Some(entry) = state.get(path) {
        let age_secs = now.saturating_sub(entry.tracked_at);
        Ok(FileStatus::Loaded {
            entry: entry.clone(),
            age_secs,
        })
    } else {
        let (size, line_count) = get_file_info(path)?;
        Ok(FileStatus::NotLoaded {
            path: path.clone(),
            size,
            line_count,
        })
    }
}

/// Resolve a path string to an absolute path
fn resolve_path(path_str: &str) -> Result<PathBuf> {
    let path = PathBuf::from(path_str);
    let resolved = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()
            .context("Failed to get current directory")?
            .join(path)
    };

    resolved
        .canonicalize()
        .with_context(|| format!("Path not found: {}", path_str))
}

/// Get file size and line count
fn get_file_info(path: &PathBuf) -> Result<(u64, usize)> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("Failed to read metadata for {}", path.display()))?;
    let size = metadata.len();

    let file =
        fs::File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
    let reader = std::io::BufReader::new(file);
    let line_count = reader.lines().count();

    Ok((size, line_count))
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Print file status to stdout
fn print_file_status(status: &FileStatus) {
    match status {
        FileStatus::Loaded { entry, age_secs } => {
            let age = format_age(*age_secs);
            println!(
                "{}: loaded {} ({} lines)",
                entry.path.display(),
                age,
                entry.line_count
            );
        }
        FileStatus::NotLoaded {
            path,
            size: _,
            line_count,
        } => {
            println!("{}: not loaded ({} lines)", path.display(), line_count);
        }
    }
}

/// Format seconds as human-readable age
fn format_age(secs: u64) -> String {
    if secs < 60 {
        format!("{}s ago", secs)
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86400)
    }
}

/// Format bytes as human-readable size
fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// Mock store for testing
    struct MockStore {
        state: RefCell<ContextState>,
        session_id: String,
        deleted: RefCell<bool>,
    }

    impl MockStore {
        fn new() -> Self {
            Self {
                state: RefCell::new(ContextState::new("mock-session".to_string())),
                session_id: "mock-session".to_string(),
                deleted: RefCell::new(false),
            }
        }

        fn with_state(state: ContextState) -> Self {
            let session_id = state.session_id.clone();
            Self {
                state: RefCell::new(state),
                session_id,
                deleted: RefCell::new(false),
            }
        }
    }

    impl ContextStore for MockStore {
        fn load(&self) -> Result<ContextState> {
            Ok(self.state.borrow().clone())
        }

        fn save(&self, state: &ContextState) -> Result<()> {
            *self.state.borrow_mut() = state.clone();
            Ok(())
        }

        fn delete(&self) -> Result<()> {
            *self.deleted.borrow_mut() = true;
            Ok(())
        }
    }

    #[test]
    fn format_age_seconds() {
        assert_eq!(format_age(0), "0s ago");
        assert_eq!(format_age(30), "30s ago");
        assert_eq!(format_age(59), "59s ago");
    }

    #[test]
    fn format_age_minutes() {
        assert_eq!(format_age(60), "1m ago");
        assert_eq!(format_age(120), "2m ago");
        assert_eq!(format_age(3599), "59m ago");
    }

    #[test]
    fn format_age_hours() {
        assert_eq!(format_age(3600), "1h ago");
        assert_eq!(format_age(7200), "2h ago");
        assert_eq!(format_age(86399), "23h ago");
    }

    #[test]
    fn format_age_days() {
        assert_eq!(format_age(86400), "1d ago");
        assert_eq!(format_age(172800), "2d ago");
    }

    #[test]
    fn format_bytes_b() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1023), "1023 B");
    }

    #[test]
    fn format_bytes_kb() {
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(2048), "2.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
    }

    #[test]
    fn format_bytes_mb() {
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(2 * 1024 * 1024), "2.0 MB");
    }

    #[test]
    fn get_file_status_loaded() {
        let mut state = ContextState::new("s".to_string());
        state.track(ContextEntry::with_timestamp(
            PathBuf::from("/test.rs"),
            100,
            10,
            1000,
        ));

        let status = get_file_status(&state, &PathBuf::from("/test.rs"), 1060).unwrap();
        if let FileStatus::Loaded { entry, age_secs } = status {
            assert_eq!(entry.size, 100);
            assert_eq!(entry.line_count, 10);
            assert_eq!(age_secs, 60);
        } else {
            panic!("Expected Loaded");
        }
    }

    #[test]
    fn get_file_status_not_loaded() {
        let state = ContextState::new("s".to_string());
        // Use Cargo.toml which we know exists
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let status = get_file_status(&state, &path, 1000).unwrap();
        assert!(matches!(status, FileStatus::NotLoaded { .. }));
    }

    #[test]
    fn clear_with_store_deletes() {
        let store = MockStore::new();
        clear_with_store(&store).unwrap();
        assert!(*store.deleted.borrow());
    }

    #[test]
    fn summary_with_store_empty() {
        let store = MockStore::new();
        // Just verify it doesn't panic
        summary_with_store(&store).unwrap();
    }

    #[test]
    fn summary_with_store_with_entries() {
        let mut state = ContextState::new("test".to_string());
        state.track(ContextEntry::with_timestamp(
            PathBuf::from("/a.rs"),
            100,
            10,
            1000,
        ));
        state.track(ContextEntry::with_timestamp(
            PathBuf::from("/b.rs"),
            200,
            20,
            2000,
        ));
        let store = MockStore::with_state(state);
        summary_with_store(&store).unwrap();
    }

    #[test]
    fn track_with_store_real_file() {
        let store = MockStore::new();
        let cargo_toml = env!("CARGO_MANIFEST_DIR").to_string() + "/Cargo.toml";
        track_with_store(&store, &[cargo_toml]).unwrap();

        let state = store.load().unwrap();
        assert_eq!(state.file_count(), 1);
    }

    #[test]
    fn check_with_store_real_file() {
        let mut state = ContextState::new("test".to_string());
        let cargo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        state.track(ContextEntry::with_timestamp(
            cargo_path.clone(),
            100,
            10,
            current_timestamp() - 60,
        ));
        let store = MockStore::with_state(state);

        check_with_store(&store, &[cargo_path.to_string_lossy().to_string()]).unwrap();
    }

    #[test]
    fn resolve_path_absolute() {
        let result = resolve_path("/tmp").unwrap();
        assert!(result.is_absolute());
    }

    #[test]
    fn resolve_path_relative() {
        // Cargo.toml should exist in project root
        let result = resolve_path("Cargo.toml").unwrap();
        assert!(result.is_absolute());
        assert!(result.to_string_lossy().ends_with("Cargo.toml"));
    }

    #[test]
    fn resolve_path_not_found() {
        let result = resolve_path("/nonexistent/path/to/file.xyz");
        assert!(result.is_err());
    }

    #[test]
    fn get_file_info_real_file() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let (size, line_count) = get_file_info(&path).unwrap();
        assert!(size > 0);
        assert!(line_count > 0);
    }

    #[test]
    fn get_file_info_missing_file() {
        let path = PathBuf::from("/nonexistent/file.txt");
        let result = get_file_info(&path);
        assert!(result.is_err());
    }

    #[test]
    fn current_timestamp_returns_value() {
        let ts = current_timestamp();
        // Should be a reasonable Unix timestamp (after 2020)
        assert!(ts > 1577836800);
    }

    #[test]
    fn mock_store_load_save() {
        let store = MockStore::new();
        let mut state = store.load().unwrap();
        state.track(ContextEntry::new(PathBuf::from("/test.rs"), 100, 10));
        store.save(&state).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.file_count(), 1);
    }

    #[test]
    fn mock_store_session_id() {
        let store = MockStore::new();
        assert_eq!(store.session_id, "mock-session");
    }
}
