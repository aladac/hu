use super::super::types::TestFailure;

/// Extract test failures from logs (RSpec format)
pub fn parse_test_failures(logs: &str) -> Vec<TestFailure> {
    let mut failures = Vec::new();

    // Collect failure error messages in order
    let mut error_messages: Vec<String> = Vec::new();

    // Find the Failures section and parse each failure block
    if let Some(failures_start) = logs.find("Failures:") {
        let failures_end = logs.find("Failed examples:").unwrap_or(logs.len());
        let failures_section = &logs[failures_start..failures_end];

        // Split by numbered failure pattern "N) description"
        let block_starts: Vec<usize> = regex::Regex::new(r"\d+\)\s+\S")
            .ok()
            .map(|re| re.find_iter(failures_section).map(|m| m.start()).collect())
            .unwrap_or_default();

        let mut positions = block_starts.clone();
        positions.push(failures_section.len());

        for i in 0..block_starts.len() {
            let block = &failures_section[positions[i]..positions[i + 1]];

            // Extract error: code line after Failure/Error: and the error message on next line
            if let Some(fe_idx) = block.find("Failure/Error:") {
                let after_fe = &block[fe_idx..];
                let lines: Vec<String> = after_fe
                    .lines()
                    .map(clean_ci_line)
                    .filter(|l| !l.is_empty())
                    .take(4)
                    .collect();

                // lines[0] = "Failure/Error: <code>"
                // lines[1] = "<error message>" or "# <stack trace>"
                let code_line = lines
                    .first()
                    .map(|l| l.strip_prefix("Failure/Error:").unwrap_or(l).trim())
                    .unwrap_or("");
                let error_msg = lines.get(1).map(|s| s.as_str()).unwrap_or("");

                let error_text = if error_msg.is_empty() || error_msg.starts_with("# ") {
                    code_line.to_string()
                } else {
                    format!("{}\n{}", code_line, error_msg)
                };

                error_messages.push(error_text);
            }
        }
    }

    // Extract failed examples from the "Failed examples:" section
    // Format: rspec ./spec/helpers/prices_api_helper_spec.rb:289 # description
    let failed_examples_re = regex::Regex::new(r"rspec\s+(\./spec/[^\s]+:\d+)").ok();

    if let Some(re) = &failed_examples_re {
        for (i, cap) in re.captures_iter(logs).enumerate() {
            let spec_file = cap.get(1).map(|m| m.as_str()).unwrap_or("");

            // Get error message by index (failures appear in same order)
            let failure_text = error_messages
                .get(i)
                .cloned()
                .unwrap_or_else(|| "Test failed".to_string());

            // Avoid duplicates
            if !failures
                .iter()
                .any(|f: &TestFailure| f.spec_file == spec_file)
            {
                failures.push(TestFailure {
                    spec_file: spec_file.to_string(),
                    failure_text,
                });
            }
        }
    }

    failures
}

/// Clean up CI log line by removing timestamp prefix
pub(super) fn clean_ci_line(line: &str) -> String {
    // Remove timestamp prefix like "2026-01-27T18:51:46.1029380Z"
    let re = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}T[\d:.]+Z\s*").ok();
    if let Some(re) = re {
        re.replace(line, "").trim().to_string()
    } else {
        line.trim().to_string()
    }
}
