//! Language-specific signature extraction for code search results.
//!
//! Extracts function/class/struct signatures from source code lines
//! to provide cleaner search output.

use regex::Regex;
use std::path::Path;

/// Try to extract function/method signature from a line based on file extension.
pub fn extract_signature(line: &str, file: &str) -> Option<String> {
    let trimmed = line.trim();
    let ext = Path::new(file)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match ext {
        "rs" => extract_rust_signature(trimmed),
        "py" => extract_python_signature(trimmed),
        "js" | "ts" | "jsx" | "tsx" => extract_js_signature(trimmed),
        "rb" => extract_ruby_signature(trimmed),
        "go" => extract_go_signature(trimmed),
        _ => None,
    }
}

/// Extract Rust function/struct signature
fn extract_rust_signature(line: &str) -> Option<String> {
    // fn name(...) -> Type
    if let Some(caps) =
        Regex::new(r"^(pub\s+)?(async\s+)?fn\s+(\w+)\s*(<[^>]+>)?\s*\([^)]*\)(\s*->\s*[^{]+)?")
            .ok()?
            .captures(line)
    {
        return Some(
            caps.get(0)?
                .as_str()
                .trim_end_matches('{')
                .trim()
                .to_string(),
        );
    }

    // struct/enum/impl
    if let Some(caps) = Regex::new(r"^(pub\s+)?(struct|enum|impl|trait)\s+(\w+)(<[^>]+>)?")
        .ok()?
        .captures(line)
    {
        return Some(caps.get(0)?.as_str().to_string());
    }

    None
}

/// Extract Python function/class signature
fn extract_python_signature(line: &str) -> Option<String> {
    // def name(...):
    if let Some(caps) = Regex::new(r"^(async\s+)?def\s+(\w+)\s*\([^)]*\)(\s*->\s*[^:]+)?:")
        .ok()?
        .captures(line)
    {
        return Some(caps.get(0)?.as_str().trim_end_matches(':').to_string());
    }

    // class Name:
    if let Some(caps) = Regex::new(r"^class\s+(\w+)(\([^)]*\))?:")
        .ok()?
        .captures(line)
    {
        return Some(caps.get(0)?.as_str().trim_end_matches(':').to_string());
    }

    None
}

/// Extract JavaScript/TypeScript function signature
fn extract_js_signature(line: &str) -> Option<String> {
    // function name(...)
    if let Some(caps) =
        Regex::new(r"^(export\s+)?(async\s+)?function\s+(\w+)\s*(<[^>]+>)?\s*\([^)]*\)")
            .ok()?
            .captures(line)
    {
        return Some(caps.get(0)?.as_str().to_string());
    }

    // const name = (...) =>
    if let Some(caps) =
        Regex::new(r"^(export\s+)?(const|let|var)\s+(\w+)\s*=\s*(async\s+)?\([^)]*\)\s*=>")
            .ok()?
            .captures(line)
    {
        return Some(
            caps.get(0)?
                .as_str()
                .trim_end_matches("=>")
                .trim()
                .to_string(),
        );
    }

    // class Name
    if let Some(caps) = Regex::new(r"^(export\s+)?class\s+(\w+)(\s+extends\s+\w+)?")
        .ok()?
        .captures(line)
    {
        return Some(caps.get(0)?.as_str().to_string());
    }

    None
}

/// Extract Ruby method/class signature
fn extract_ruby_signature(line: &str) -> Option<String> {
    // def name(...)
    if let Some(caps) = Regex::new(r"^def\s+(\w+[?!=]?)(\([^)]*\))?")
        .ok()?
        .captures(line)
    {
        return Some(caps.get(0)?.as_str().to_string());
    }

    // class Name
    if let Some(caps) = Regex::new(r"^class\s+(\w+)(\s*<\s*\w+)?")
        .ok()?
        .captures(line)
    {
        return Some(caps.get(0)?.as_str().to_string());
    }

    None
}

/// Extract Go function signature
fn extract_go_signature(line: &str) -> Option<String> {
    // func name(...)
    if let Some(caps) =
        Regex::new(r"^func\s+(\([^)]+\)\s+)?(\w+)\s*\([^)]*\)(\s*\([^)]*\)|\s*\w+)?")
            .ok()?
            .captures(line)
    {
        return Some(caps.get(0)?.as_str().to_string());
    }

    // type Name struct/interface
    if let Some(caps) = Regex::new(r"^type\s+(\w+)\s+(struct|interface)")
        .ok()?
        .captures(line)
    {
        return Some(caps.get(0)?.as_str().to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_rust_fn_signature() {
        let sig = extract_rust_signature("pub fn foo(x: i32) -> String {").unwrap();
        assert_eq!(sig, "pub fn foo(x: i32) -> String");
    }

    #[test]
    fn extract_rust_async_fn_signature() {
        let sig = extract_rust_signature("pub async fn fetch() -> Result<()> {").unwrap();
        assert_eq!(sig, "pub async fn fetch() -> Result<()>");
    }

    #[test]
    fn extract_rust_struct_signature() {
        let sig = extract_rust_signature("pub struct Config<T> {").unwrap();
        assert_eq!(sig, "pub struct Config<T>");
    }

    #[test]
    fn extract_python_def_signature() {
        let sig = extract_python_signature("def process(data: list) -> dict:").unwrap();
        assert_eq!(sig, "def process(data: list) -> dict");
    }

    #[test]
    fn extract_python_class_signature() {
        let sig = extract_python_signature("class Handler(BaseHandler):").unwrap();
        assert_eq!(sig, "class Handler(BaseHandler)");
    }

    #[test]
    fn extract_python_async_def() {
        let sig = extract_python_signature("async def fetch_data(url: str) -> dict:").unwrap();
        assert!(sig.contains("async def fetch_data"));
    }

    #[test]
    fn extract_python_signature_no_match() {
        let result = extract_python_signature("just some regular text");
        assert!(result.is_none());
    }

    #[test]
    fn extract_js_function_signature() {
        let sig = extract_js_signature("export async function fetchData(url) {").unwrap();
        assert_eq!(sig, "export async function fetchData(url)");
    }

    #[test]
    fn extract_js_arrow_signature() {
        let sig = extract_js_signature("const handler = async (req, res) =>").unwrap();
        assert_eq!(sig, "const handler = async (req, res)");
    }

    #[test]
    fn extract_js_class_signature() {
        let sig = extract_js_signature("export class UserService extends BaseService {").unwrap();
        assert!(sig.contains("class UserService"));
        assert!(sig.contains("extends BaseService"));
    }

    #[test]
    fn extract_js_signature_no_match() {
        let result = extract_js_signature("console.log('hello')");
        assert!(result.is_none());
    }

    #[test]
    fn extract_ruby_def_signature() {
        let sig = extract_ruby_signature("def process(data)").unwrap();
        assert_eq!(sig, "def process(data)");
    }

    #[test]
    fn extract_ruby_predicate_signature() {
        let sig = extract_ruby_signature("def valid?").unwrap();
        assert_eq!(sig, "def valid?");
    }

    #[test]
    fn extract_ruby_class_with_inheritance() {
        let sig = extract_ruby_signature("class User < ActiveRecord::Base").unwrap();
        assert!(sig.contains("class User"));
    }

    #[test]
    fn extract_ruby_signature_no_match() {
        let result = extract_ruby_signature("puts 'hello world'");
        assert!(result.is_none());
    }

    #[test]
    fn extract_go_func_signature() {
        let sig =
            extract_go_signature("func (s *Server) Handle(w http.ResponseWriter, r *http.Request)")
                .unwrap();
        assert!(sig.contains("func"));
        assert!(sig.contains("Handle"));
    }

    #[test]
    fn extract_go_type_interface() {
        let sig = extract_go_signature("type Handler interface {").unwrap();
        assert_eq!(sig, "type Handler interface");
    }

    #[test]
    fn extract_go_signature_no_match() {
        let result = extract_go_signature("fmt.Println(\"hello\")");
        assert!(result.is_none());
    }

    #[test]
    fn extract_signature_by_extension() {
        let sig = extract_signature("pub fn test() {", "foo.rs").unwrap();
        assert!(sig.contains("fn test"));

        let sig = extract_signature("def test():", "foo.py").unwrap();
        assert!(sig.contains("def test"));
    }

    #[test]
    fn extract_signature_unknown_extension() {
        let result = extract_signature("some random line", "file.xyz");
        assert!(result.is_none());
    }
}
