use super::docs_index::DocsIndex;

/// Search result
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    /// File path (relative)
    pub file: String,
    /// Section heading
    pub heading: String,
    /// Section level
    pub level: u8,
    /// Start line in file
    pub start_line: usize,
    /// End line in file
    pub end_line: usize,
    /// Match score (higher is better)
    pub score: u32,
}

/// Search the index for matching sections
pub fn search_index(index: &DocsIndex, query: &str) -> Vec<SearchResult> {
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();
    let mut results = Vec::new();

    for (path, file_index) in &index.files {
        for section in &file_index.sections {
            if let Some(score) = match_score(&section.heading, &query_lower, &query_words) {
                results.push(SearchResult {
                    file: path.clone(),
                    heading: section.heading.clone(),
                    level: section.level,
                    start_line: section.start_line,
                    end_line: section.end_line,
                    score,
                });
            }
        }
    }

    // Sort by score (descending)
    results.sort_by(|a, b| b.score.cmp(&a.score));

    results
}

/// Calculate match score for a heading against a query
fn match_score(heading: &str, query_lower: &str, query_words: &[&str]) -> Option<u32> {
    let heading_lower = heading.to_lowercase();

    // Exact match (highest priority)
    if heading_lower == query_lower {
        return Some(1000);
    }

    // Contains full query
    if heading_lower.contains(query_lower) {
        return Some(500);
    }

    // Word matching
    let heading_words: Vec<&str> = heading_lower.split_whitespace().collect();
    let mut matched_words = 0;

    for qw in query_words {
        for hw in &heading_words {
            if hw.contains(qw) || qw.contains(hw) {
                matched_words += 1;
                break;
            }
        }
    }

    if matched_words > 0 {
        // Score based on percentage of query words matched
        let score = (matched_words * 100) / query_words.len().max(1);
        return Some(score as u32);
    }

    None
}

/// Format search results for display
pub fn format_results(results: &[SearchResult], limit: Option<usize>) -> String {
    if results.is_empty() {
        return "No matching sections found".to_string();
    }

    let results = if let Some(n) = limit {
        &results[..n.min(results.len())]
    } else {
        results
    };

    let mut output = Vec::new();

    for result in results {
        let level_marker = "#".repeat(result.level as usize);
        output.push(format!(
            "{} {} ({}:L{}-{})",
            level_marker, result.heading, result.file, result.start_line, result.end_line
        ));
    }

    output.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::docs_index::{FileIndex, Section};

    fn test_index() -> DocsIndex {
        let mut index = DocsIndex::new("./".to_string());

        let mut readme = FileIndex::new("README.md".to_string(), 50);
        readme.sections.push(Section {
            heading: "Getting Started".to_string(),
            level: 1,
            start_line: 1,
            end_line: 20,
        });
        readme.sections.push(Section {
            heading: "Installation".to_string(),
            level: 2,
            start_line: 5,
            end_line: 15,
        });
        readme.sections.push(Section {
            heading: "Configuration".to_string(),
            level: 2,
            start_line: 15,
            end_line: 20,
        });

        let mut api = FileIndex::new("api.md".to_string(), 100);
        api.sections.push(Section {
            heading: "API Reference".to_string(),
            level: 1,
            start_line: 1,
            end_line: 100,
        });
        api.sections.push(Section {
            heading: "Getting Started with API".to_string(),
            level: 2,
            start_line: 10,
            end_line: 50,
        });

        index.add_file(readme);
        index.add_file(api);

        index
    }

    #[test]
    fn search_exact_match() {
        let index = test_index();
        let results = search_index(&index, "Installation");
        assert!(!results.is_empty());
        assert_eq!(results[0].heading, "Installation");
        assert_eq!(results[0].score, 1000);
    }

    #[test]
    fn search_case_insensitive() {
        let index = test_index();
        let results = search_index(&index, "installation");
        assert!(!results.is_empty());
        assert_eq!(results[0].heading, "Installation");
    }

    #[test]
    fn search_contains() {
        let index = test_index();
        let results = search_index(&index, "Config");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.heading == "Configuration"));
    }

    #[test]
    fn search_word_match() {
        let index = test_index();
        let results = search_index(&index, "Getting");
        assert_eq!(results.len(), 2); // "Getting Started" and "Getting Started with API"
    }

    #[test]
    fn search_no_match() {
        let index = test_index();
        let results = search_index(&index, "nonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn search_sorted_by_score() {
        let index = test_index();
        let results = search_index(&index, "Installation");
        // Exact match should be first
        assert!(results[0].score >= results.last().map(|r| r.score).unwrap_or(0));
    }

    #[test]
    fn search_result_clone() {
        let result = SearchResult {
            file: "test.md".to_string(),
            heading: "Test".to_string(),
            level: 1,
            start_line: 1,
            end_line: 10,
            score: 100,
        };
        let cloned = result.clone();
        assert_eq!(result, cloned);
    }

    #[test]
    fn search_result_debug() {
        let result = SearchResult {
            file: "test.md".to_string(),
            heading: "Test".to_string(),
            level: 1,
            start_line: 1,
            end_line: 10,
            score: 100,
        };
        let debug = format!("{:?}", result);
        assert!(debug.contains("SearchResult"));
    }

    #[test]
    fn match_score_exact() {
        let score = match_score("Installation", "installation", &["installation"]);
        assert_eq!(score, Some(1000));
    }

    #[test]
    fn match_score_contains() {
        let score = match_score("Configuration Options", "config", &["config"]);
        assert_eq!(score, Some(500));
    }

    #[test]
    fn match_score_word_partial() {
        // Test that partial word matches work when heading word contains query word
        // or query word contains heading word
        // "started" contains "start" - heading word contains query word
        let score = match_score("Started Guide", "start", &["start"]);
        // But "started guide" also contains "start" as substring, so it gets 500
        assert_eq!(score, Some(500));

        // Test case where query word contains heading word (qw.contains(hw))
        // "testing" contains "test"
        let score2 = match_score("Test Results", "testing", &["testing"]);
        // "test results" does NOT contain "testing", so it falls through to word matching
        // Word matching: "testing".contains("test")? Yes
        assert!(score2.is_some());
        assert!(score2.unwrap() < 500);

        // Another case: heading contains partial word but not as substring
        let score3 = match_score("API Docs", "apis", &["apis"]);
        // "api docs" does NOT contain "apis", but "apis".contains("api")? Yes
        assert!(score3.is_some());
        assert!(score3.unwrap() < 500);
    }

    #[test]
    fn match_score_no_match() {
        let score = match_score("Installation", "api", &["api"]);
        assert!(score.is_none());
    }

    #[test]
    fn format_results_empty() {
        let results: Vec<SearchResult> = vec![];
        let output = format_results(&results, None);
        assert_eq!(output, "No matching sections found");
    }

    #[test]
    fn format_results_single() {
        let results = vec![SearchResult {
            file: "test.md".to_string(),
            heading: "Test Section".to_string(),
            level: 2,
            start_line: 5,
            end_line: 15,
            score: 100,
        }];
        let output = format_results(&results, None);
        assert!(output.contains("## Test Section"));
        assert!(output.contains("test.md:L5-15"));
    }

    #[test]
    fn format_results_multiple() {
        let results = vec![
            SearchResult {
                file: "a.md".to_string(),
                heading: "First".to_string(),
                level: 1,
                start_line: 1,
                end_line: 10,
                score: 100,
            },
            SearchResult {
                file: "b.md".to_string(),
                heading: "Second".to_string(),
                level: 2,
                start_line: 5,
                end_line: 20,
                score: 50,
            },
        ];
        let output = format_results(&results, None);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn format_results_with_limit() {
        let results = vec![
            SearchResult {
                file: "a.md".to_string(),
                heading: "First".to_string(),
                level: 1,
                start_line: 1,
                end_line: 10,
                score: 100,
            },
            SearchResult {
                file: "b.md".to_string(),
                heading: "Second".to_string(),
                level: 2,
                start_line: 5,
                end_line: 20,
                score: 50,
            },
        ];
        let output = format_results(&results, Some(1));
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 1);
    }
}
