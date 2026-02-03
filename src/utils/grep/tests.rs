use super::*;

#[test]
fn glob_matches_extension() {
    assert!(glob_matches("foo.rs", "*.rs"));
    assert!(glob_matches("bar.rs", "*.rs"));
    assert!(!glob_matches("foo.py", "*.rs"));
}

#[test]
fn glob_matches_recursive() {
    assert!(glob_matches("foo.rs", "**/*.rs"));
}

#[test]
fn glob_matches_exact() {
    assert!(glob_matches("Cargo.toml", "Cargo.toml"));
    assert!(!glob_matches("cargo.toml", "Cargo.toml"));
}

#[test]
fn glob_matches_wildcard() {
    assert!(glob_matches("test_foo.rs", "test_*.rs"));
    assert!(!glob_matches("foo_test.rs", "test_*.rs"));
}

#[test]
fn is_ignored_dir_common() {
    assert!(is_ignored_dir("node_modules"));
    assert!(is_ignored_dir("target"));
    assert!(is_ignored_dir(".git"));
    assert!(!is_ignored_dir("src"));
    assert!(!is_ignored_dir("lib"));
}

#[test]
fn is_binary_extension_images() {
    assert!(is_binary_extension("png"));
    assert!(is_binary_extension("jpg"));
    assert!(is_binary_extension("gif"));
}

#[test]
fn is_binary_extension_archives() {
    assert!(is_binary_extension("zip"));
    assert!(is_binary_extension("tar"));
    assert!(is_binary_extension("gz"));
}

#[test]
fn is_binary_extension_code() {
    assert!(!is_binary_extension("rs"));
    assert!(!is_binary_extension("py"));
    assert!(!is_binary_extension("js"));
}

#[test]
fn format_matches_refs_mode() {
    let matches = vec![GrepMatch {
        file: "src/main.rs".to_string(),
        line_num: 42,
        content: "    let x = 1;".to_string(),
        match_count: 1,
    }];
    let args = GrepArgs {
        pattern: "x".to_string(),
        path: ".".to_string(),
        refs: true,
        unique: false,
        ranked: false,
        limit: None,
        signature: false,
        glob: None,
        ignore_case: false,
        hidden: false,
    };
    let output = format_matches(&matches, &args);
    assert_eq!(output, "src/main.rs:42");
}

#[test]
fn format_matches_full_mode() {
    let matches = vec![GrepMatch {
        file: "src/main.rs".to_string(),
        line_num: 42,
        content: "    let x = 1;".to_string(),
        match_count: 1,
    }];
    let args = GrepArgs {
        pattern: "x".to_string(),
        path: ".".to_string(),
        refs: false,
        unique: false,
        ranked: false,
        limit: None,
        signature: false,
        glob: None,
        ignore_case: false,
        hidden: false,
    };
    let output = format_matches(&matches, &args);
    assert_eq!(output, "src/main.rs:42: let x = 1;");
}

#[test]
fn dedupe_matches_combines_counts() {
    let matches = vec![
        GrepMatch {
            file: "a.rs".to_string(),
            line_num: 1,
            content: "let x = 1;".to_string(),
            match_count: 1,
        },
        GrepMatch {
            file: "b.rs".to_string(),
            line_num: 5,
            content: "let x = 1;".to_string(),
            match_count: 2,
        },
    ];
    let deduped = dedupe_matches(matches);
    assert_eq!(deduped.len(), 1);
    assert_eq!(deduped[0].match_count, 3); // 1 + 2
}

#[test]
fn rank_matches_by_count() {
    let mut matches = vec![
        GrepMatch {
            file: "a.rs".to_string(),
            line_num: 1,
            content: "one match".to_string(),
            match_count: 1,
        },
        GrepMatch {
            file: "b.rs".to_string(),
            line_num: 2,
            content: "three matches".to_string(),
            match_count: 3,
        },
    ];
    rank_matches(&mut matches);
    assert_eq!(matches[0].match_count, 3); // Higher count first
}

#[test]
fn search_files_respects_limit() {
    let temp_dir = std::env::temp_dir().join("hu_grep_test");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    std::fs::write(temp_dir.join("a.txt"), "test line 1\ntest line 2\n").unwrap();
    std::fs::write(temp_dir.join("b.txt"), "test line 3\n").unwrap();

    let args = GrepArgs {
        pattern: "test".to_string(),
        path: temp_dir.to_str().unwrap().to_string(),
        refs: false,
        unique: false,
        ranked: false,
        limit: Some(2),
        signature: false,
        glob: None,
        ignore_case: false,
        hidden: false,
    };

    let matches = search_files(&args).unwrap();
    assert_eq!(matches.len(), 2);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn search_files_respects_glob() {
    let temp_dir = std::env::temp_dir().join("hu_grep_glob_test");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    std::fs::write(temp_dir.join("foo.rs"), "test\n").unwrap();
    std::fs::write(temp_dir.join("bar.py"), "test\n").unwrap();

    let args = GrepArgs {
        pattern: "test".to_string(),
        path: temp_dir.to_str().unwrap().to_string(),
        refs: false,
        unique: false,
        ranked: false,
        limit: None,
        signature: false,
        glob: Some("*.rs".to_string()),
        ignore_case: false,
        hidden: false,
    };

    let matches = search_files(&args).unwrap();
    assert_eq!(matches.len(), 1);
    assert!(matches[0].file.ends_with("foo.rs"));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn format_matches_signature_mode() {
    let matches = vec![GrepMatch {
        file: "src/main.rs".to_string(),
        line_num: 42,
        content: "pub fn process() {".to_string(),
        match_count: 1,
    }];
    let args = GrepArgs {
        pattern: "process".to_string(),
        path: ".".to_string(),
        refs: false,
        unique: false,
        ranked: false,
        limit: None,
        signature: true,
        glob: None,
        ignore_case: false,
        hidden: false,
    };
    let output = format_matches(&matches, &args);
    assert!(output.contains("pub fn process()"));
    assert!(!output.contains("{")); // signature strips the brace
}

#[test]
fn format_matches_signature_no_match() {
    // When line doesn't match signature pattern, falls back to trimmed content
    let matches = vec![GrepMatch {
        file: "src/main.rs".to_string(),
        line_num: 42,
        content: "    let x = 1;".to_string(),
        match_count: 1,
    }];
    let args = GrepArgs {
        pattern: "x".to_string(),
        path: ".".to_string(),
        refs: false,
        unique: false,
        ranked: false,
        limit: None,
        signature: true,
        glob: None,
        ignore_case: false,
        hidden: false,
    };
    let output = format_matches(&matches, &args);
    assert!(output.contains("let x = 1;"));
}

#[test]
fn grep_match_debug() {
    let m = GrepMatch {
        file: "test.rs".to_string(),
        line_num: 1,
        content: "test".to_string(),
        match_count: 1,
    };
    let debug = format!("{:?}", m);
    assert!(debug.contains("GrepMatch"));
}

#[test]
fn grep_match_clone() {
    let m = GrepMatch {
        file: "test.rs".to_string(),
        line_num: 1,
        content: "test".to_string(),
        match_count: 1,
    };
    let cloned = m.clone();
    assert_eq!(cloned.file, m.file);
    assert_eq!(cloned.line_num, m.line_num);
}

#[test]
fn search_files_with_unique() {
    let temp_dir = std::env::temp_dir().join("hu_grep_unique_test");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    std::fs::write(temp_dir.join("a.txt"), "let x = 1;\n").unwrap();
    std::fs::write(temp_dir.join("b.txt"), "let x = 1;\n").unwrap();

    let args = GrepArgs {
        pattern: "let".to_string(),
        path: temp_dir.to_str().unwrap().to_string(),
        refs: false,
        unique: true,
        ranked: false,
        limit: None,
        signature: false,
        glob: None,
        ignore_case: false,
        hidden: false,
    };

    let matches = search_files(&args).unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].match_count, 2);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn search_files_with_ranked() {
    let temp_dir = std::env::temp_dir().join("hu_grep_ranked_test");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    std::fs::write(temp_dir.join("a.txt"), "test\n").unwrap();
    std::fs::write(temp_dir.join("b.txt"), "test test test\n").unwrap();

    let args = GrepArgs {
        pattern: "test".to_string(),
        path: temp_dir.to_str().unwrap().to_string(),
        refs: false,
        unique: false,
        ranked: true,
        limit: None,
        signature: false,
        glob: None,
        ignore_case: false,
        hidden: false,
    };

    let matches = search_files(&args).unwrap();
    assert!(matches[0].match_count >= matches[1].match_count);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn collect_matches_skips_hidden() {
    let temp_dir = std::env::temp_dir().join("hu_grep_hidden_test");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::create_dir_all(temp_dir.join(".hidden")).unwrap();

    std::fs::write(temp_dir.join("visible.txt"), "test\n").unwrap();
    std::fs::write(temp_dir.join(".hidden").join("secret.txt"), "test\n").unwrap();

    let args = GrepArgs {
        pattern: "test".to_string(),
        path: temp_dir.to_str().unwrap().to_string(),
        refs: false,
        unique: false,
        ranked: false,
        limit: None,
        signature: false,
        glob: None,
        ignore_case: false,
        hidden: false,
    };

    let matches = search_files(&args).unwrap();
    assert_eq!(matches.len(), 1);
    assert!(matches[0].file.contains("visible"));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn collect_matches_includes_hidden_when_requested() {
    let temp_dir = std::env::temp_dir().join("hu_grep_hidden_incl_test");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    std::fs::write(temp_dir.join(".hidden_file.txt"), "test\n").unwrap();
    std::fs::write(temp_dir.join("visible.txt"), "test\n").unwrap();

    let args = GrepArgs {
        pattern: "test".to_string(),
        path: temp_dir.to_str().unwrap().to_string(),
        refs: false,
        unique: false,
        ranked: false,
        limit: None,
        signature: false,
        glob: None,
        ignore_case: false,
        hidden: true,
    };

    let matches = search_files(&args).unwrap();
    assert_eq!(matches.len(), 2);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn search_files_skips_ignored_dirs() {
    let temp_dir = std::env::temp_dir().join("hu_grep_ignored_test");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::create_dir_all(temp_dir.join("node_modules")).unwrap();

    std::fs::write(temp_dir.join("app.js"), "test\n").unwrap();
    std::fs::write(temp_dir.join("node_modules").join("dep.js"), "test\n").unwrap();

    let args = GrepArgs {
        pattern: "test".to_string(),
        path: temp_dir.to_str().unwrap().to_string(),
        refs: false,
        unique: false,
        ranked: false,
        limit: None,
        signature: false,
        glob: None,
        ignore_case: false,
        hidden: false,
    };

    let matches = search_files(&args).unwrap();
    assert_eq!(matches.len(), 1);
    assert!(matches[0].file.contains("app.js"));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn search_files_single_file_path() {
    let temp_dir = std::env::temp_dir().join("hu_grep_single_file_test");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    let file_path = temp_dir.join("single.txt");
    std::fs::write(&file_path, "test line\n").unwrap();

    let args = GrepArgs {
        pattern: "test".to_string(),
        path: file_path.to_str().unwrap().to_string(),
        refs: false,
        unique: false,
        ranked: false,
        limit: None,
        signature: false,
        glob: None,
        ignore_case: false,
        hidden: false,
    };

    let matches = search_files(&args).unwrap();
    assert_eq!(matches.len(), 1);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn search_files_nonexistent_path() {
    let args = GrepArgs {
        pattern: "test".to_string(),
        path: "/nonexistent/path/12345".to_string(),
        refs: false,
        unique: false,
        ranked: false,
        limit: None,
        signature: false,
        glob: None,
        ignore_case: false,
        hidden: false,
    };

    let matches = search_files(&args).unwrap();
    assert!(matches.is_empty());
}

#[test]
fn search_files_invalid_regex() {
    let args = GrepArgs {
        pattern: "[invalid".to_string(),
        path: ".".to_string(),
        refs: false,
        unique: false,
        ranked: false,
        limit: None,
        signature: false,
        glob: None,
        ignore_case: false,
        hidden: false,
    };

    let result = search_files(&args);
    assert!(result.is_err());
}

#[test]
fn search_files_case_insensitive() {
    let temp_dir = std::env::temp_dir().join("hu_grep_case_test");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    std::fs::write(temp_dir.join("test.txt"), "Hello HELLO hello\n").unwrap();

    let args_sensitive = GrepArgs {
        pattern: "Hello".to_string(),
        path: temp_dir.to_str().unwrap().to_string(),
        refs: false,
        unique: false,
        ranked: false,
        limit: None,
        signature: false,
        glob: None,
        ignore_case: false,
        hidden: false,
    };

    let matches_sensitive = search_files(&args_sensitive).unwrap();
    assert_eq!(matches_sensitive[0].match_count, 1);

    let args_insensitive = GrepArgs {
        pattern: "Hello".to_string(),
        path: temp_dir.to_str().unwrap().to_string(),
        refs: false,
        unique: false,
        ranked: false,
        limit: None,
        signature: false,
        glob: None,
        ignore_case: true,
        hidden: false,
    };

    let matches_insensitive = search_files(&args_insensitive).unwrap();
    assert_eq!(matches_insensitive[0].match_count, 3);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn should_search_file_binary_extension() {
    let path = std::path::Path::new("image.png");
    assert!(!should_search_file(path, None));
}

#[test]
fn should_search_file_text_no_glob() {
    let path = std::path::Path::new("file.txt");
    assert!(should_search_file(path, None));
}
