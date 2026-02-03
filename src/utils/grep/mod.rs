use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::cli::GrepArgs;
use super::signature::extract_signature;

#[cfg(test)]
mod tests;

/// A single grep match
#[derive(Debug, Clone)]
pub struct GrepMatch {
    pub file: String,
    pub line_num: usize,
    pub content: String,
    pub match_count: usize,
}

/// Handle the `hu utils grep` command
pub fn run(args: GrepArgs) -> Result<()> {
    let matches = search_files(&args)?;

    if matches.is_empty() {
        eprintln!("No matches found.");
        return Ok(());
    }

    let output = format_matches(&matches, &args);
    println!("{}", output);

    Ok(())
}

/// Search files for pattern
pub fn search_files(args: &GrepArgs) -> Result<Vec<GrepMatch>> {
    let re = if args.ignore_case {
        Regex::new(&format!("(?i){}", &args.pattern))
    } else {
        Regex::new(&args.pattern)
    }
    .with_context(|| format!("Invalid regex pattern: {}", args.pattern))?;

    let glob_pattern = args.glob.as_deref();
    let mut matches = Vec::new();

    collect_matches(&args.path, &re, glob_pattern, args.hidden, &mut matches)?;

    // Apply post-processing
    let mut matches = if args.unique {
        dedupe_matches(matches)
    } else {
        matches
    };

    if args.ranked {
        rank_matches(&mut matches);
    }

    if let Some(limit) = args.limit {
        matches.truncate(limit);
    }

    Ok(matches)
}

/// Recursively collect matches from files
fn collect_matches(
    path: &str,
    re: &Regex,
    glob_pattern: Option<&str>,
    include_hidden: bool,
    matches: &mut Vec<GrepMatch>,
) -> Result<()> {
    let path = Path::new(path);

    if path.is_file() {
        if should_search_file(path, glob_pattern) {
            search_file(path, re, matches)?;
        }
        return Ok(());
    }

    if !path.is_dir() {
        return Ok(());
    }

    let entries =
        fs::read_dir(path).with_context(|| format!("Failed to read directory: {:?}", path))?;

    for entry in entries.flatten() {
        let entry_path = entry.path();
        let file_name = entry_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Skip hidden files unless requested
        if !include_hidden && file_name.starts_with('.') {
            continue;
        }

        // Skip common non-code directories
        if entry_path.is_dir() && is_ignored_dir(file_name) {
            continue;
        }

        if entry_path.is_dir() {
            collect_matches(
                entry_path.to_str().unwrap_or(""),
                re,
                glob_pattern,
                include_hidden,
                matches,
            )?;
        } else if should_search_file(&entry_path, glob_pattern) {
            search_file(&entry_path, re, matches)?;
        }
    }

    Ok(())
}

/// Check if a directory should be ignored
fn is_ignored_dir(name: &str) -> bool {
    matches!(
        name,
        "node_modules"
            | "target"
            | ".git"
            | ".svn"
            | ".hg"
            | "__pycache__"
            | ".mypy_cache"
            | ".pytest_cache"
            | "venv"
            | ".venv"
            | "dist"
            | "build"
            | ".next"
            | ".nuxt"
    )
}

/// Check if a file matches the glob pattern
fn should_search_file(path: &Path, glob_pattern: Option<&str>) -> bool {
    // Skip binary files
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if is_binary_extension(ext) {
        return false;
    }

    // If no glob, search all text files
    let Some(pattern) = glob_pattern else {
        return true;
    };

    // Simple glob matching
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    glob_matches(file_name, pattern)
}

/// Check if extension indicates binary file
fn is_binary_extension(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "png"
            | "jpg"
            | "jpeg"
            | "gif"
            | "ico"
            | "webp"
            | "bmp"
            | "svg"
            | "pdf"
            | "zip"
            | "tar"
            | "gz"
            | "bz2"
            | "xz"
            | "7z"
            | "rar"
            | "exe"
            | "dll"
            | "so"
            | "dylib"
            | "a"
            | "o"
            | "obj"
            | "wasm"
            | "class"
            | "jar"
            | "pyc"
            | "pyo"
            | "mp3"
            | "mp4"
            | "avi"
            | "mkv"
            | "mov"
            | "wav"
            | "flac"
            | "ttf"
            | "otf"
            | "woff"
            | "woff2"
            | "eot"
            | "sqlite"
            | "db"
    )
}

/// Simple glob matching (supports * and ?)
pub fn glob_matches(name: &str, pattern: &str) -> bool {
    let pattern = pattern.trim_start_matches("**/");

    if let Some(ext) = pattern.strip_prefix("*.") {
        // Extension match: *.rs
        name.ends_with(&format!(".{}", ext))
    } else if pattern.contains('*') {
        // Convert glob to regex
        let regex_pattern = pattern
            .replace('.', "\\.")
            .replace('*', ".*")
            .replace('?', ".");
        Regex::new(&format!("^{}$", regex_pattern))
            .map(|re| re.is_match(name))
            .unwrap_or(false)
    } else {
        // Exact match
        name == pattern
    }
}

/// Search a single file for matches
fn search_file(path: &Path, re: &Regex, matches: &mut Vec<GrepMatch>) -> Result<()> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Ok(()), // Skip unreadable files
    };

    let file_str = path.to_str().unwrap_or("");

    for (line_num, line) in content.lines().enumerate() {
        let match_count = re.find_iter(line).count();
        if match_count > 0 {
            matches.push(GrepMatch {
                file: file_str.to_string(),
                line_num: line_num + 1,
                content: line.to_string(),
                match_count,
            });
        }
    }

    Ok(())
}

/// Deduplicate similar matches
fn dedupe_matches(matches: Vec<GrepMatch>) -> Vec<GrepMatch> {
    let mut seen: HashMap<String, GrepMatch> = HashMap::new();

    for m in matches {
        // Normalize content for comparison (trim, collapse whitespace)
        let normalized = m.content.split_whitespace().collect::<Vec<_>>().join(" ");

        seen.entry(normalized)
            .and_modify(|existing| existing.match_count += m.match_count)
            .or_insert(m);
    }

    seen.into_values().collect()
}

/// Rank matches by relevance (match density)
fn rank_matches(matches: &mut [GrepMatch]) {
    matches.sort_by(|a, b| {
        // Higher match count first
        b.match_count
            .cmp(&a.match_count)
            // Then shorter content (more focused)
            .then_with(|| a.content.len().cmp(&b.content.len()))
    });
}

/// Format matches for output
pub fn format_matches(matches: &[GrepMatch], args: &GrepArgs) -> String {
    let mut output = Vec::new();

    for m in matches {
        if args.refs {
            // Just file:line reference
            output.push(format!("{}:{}", m.file, m.line_num));
        } else if args.signature {
            // Try to extract function signature
            if let Some(sig) = extract_signature(&m.content, &m.file) {
                output.push(format!("{}:{}: {}", m.file, m.line_num, sig));
            } else {
                output.push(format!("{}:{}: {}", m.file, m.line_num, m.content.trim()));
            }
        } else {
            // Full match with content
            output.push(format!("{}:{}: {}", m.file, m.line_num, m.content.trim()));
        }
    }

    output.join("\n")
}
