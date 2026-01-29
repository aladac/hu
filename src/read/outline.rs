use regex::Regex;
use std::path::Path;

use super::types::{FileOutline, ItemKind, OutlineItem};

/// Extract outline from file content based on extension
pub fn extract_outline(content: &str, path: &str) -> FileOutline {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let mut outline = FileOutline::new();

    match ext {
        "rs" => extract_rust_outline(content, &mut outline),
        "py" => extract_python_outline(content, &mut outline),
        "js" | "ts" | "jsx" | "tsx" | "mjs" => extract_js_outline(content, &mut outline),
        "rb" => extract_ruby_outline(content, &mut outline),
        "go" => extract_go_outline(content, &mut outline),
        "md" | "markdown" => extract_markdown_outline(content, &mut outline),
        _ => {}
    }

    outline
}

/// Extract Rust outline (functions, structs, enums, traits, impls)
fn extract_rust_outline(content: &str, outline: &mut FileOutline) {
    let fn_re = Regex::new(
        r"^(\s*)(pub\s+)?(async\s+)?fn\s+(\w+)\s*(<[^>]+>)?\s*\([^)]*\)(\s*->\s*[^{]+)?",
    )
    .unwrap();
    let struct_re = Regex::new(r"^(\s*)(pub\s+)?struct\s+(\w+)(<[^>]+>)?").unwrap();
    let enum_re = Regex::new(r"^(\s*)(pub\s+)?enum\s+(\w+)(<[^>]+>)?").unwrap();
    let trait_re = Regex::new(r"^(\s*)(pub\s+)?trait\s+(\w+)(<[^>]+>)?").unwrap();
    let impl_re = Regex::new(r"^(\s*)impl\s*(<[^>]+>)?\s*(\w+)(<[^>]+>)?(\s+for\s+\w+)?").unwrap();
    let mod_re = Regex::new(r"^(\s*)(pub\s+)?mod\s+(\w+)").unwrap();
    let const_re = Regex::new(r"^(\s*)(pub\s+)?const\s+(\w+)").unwrap();
    let type_re = Regex::new(r"^(\s*)(pub\s+)?type\s+(\w+)").unwrap();

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1;

        if let Some(caps) = fn_re.captures(line) {
            let indent = caps.get(1).map_or(0, |m| m.as_str().len());
            let sig = caps.get(0).unwrap().as_str().trim_end_matches('{').trim();
            outline.push(OutlineItem::new(
                line_num,
                sig.to_string(),
                indent / 4,
                ItemKind::Function,
            ));
        } else if let Some(caps) = struct_re.captures(line) {
            let indent = caps.get(1).map_or(0, |m| m.as_str().len());
            let sig = caps.get(0).unwrap().as_str().trim();
            outline.push(OutlineItem::new(
                line_num,
                sig.to_string(),
                indent / 4,
                ItemKind::Struct,
            ));
        } else if let Some(caps) = enum_re.captures(line) {
            let indent = caps.get(1).map_or(0, |m| m.as_str().len());
            let sig = caps.get(0).unwrap().as_str().trim();
            outline.push(OutlineItem::new(
                line_num,
                sig.to_string(),
                indent / 4,
                ItemKind::Enum,
            ));
        } else if let Some(caps) = trait_re.captures(line) {
            let indent = caps.get(1).map_or(0, |m| m.as_str().len());
            let sig = caps.get(0).unwrap().as_str().trim();
            outline.push(OutlineItem::new(
                line_num,
                sig.to_string(),
                indent / 4,
                ItemKind::Trait,
            ));
        } else if let Some(caps) = impl_re.captures(line) {
            let indent = caps.get(1).map_or(0, |m| m.as_str().len());
            let sig = caps.get(0).unwrap().as_str().trim();
            outline.push(OutlineItem::new(
                line_num,
                sig.to_string(),
                indent / 4,
                ItemKind::Impl,
            ));
        } else if let Some(caps) = mod_re.captures(line) {
            let indent = caps.get(1).map_or(0, |m| m.as_str().len());
            let sig = caps.get(0).unwrap().as_str().trim();
            outline.push(OutlineItem::new(
                line_num,
                sig.to_string(),
                indent / 4,
                ItemKind::Module,
            ));
        } else if let Some(caps) = const_re.captures(line) {
            let indent = caps.get(1).map_or(0, |m| m.as_str().len());
            let sig = caps.get(0).unwrap().as_str().trim();
            outline.push(OutlineItem::new(
                line_num,
                sig.to_string(),
                indent / 4,
                ItemKind::Const,
            ));
        } else if let Some(caps) = type_re.captures(line) {
            let indent = caps.get(1).map_or(0, |m| m.as_str().len());
            let sig = caps.get(0).unwrap().as_str().trim();
            outline.push(OutlineItem::new(
                line_num,
                sig.to_string(),
                indent / 4,
                ItemKind::Type,
            ));
        }
    }
}

/// Extract Python outline (functions, classes)
fn extract_python_outline(content: &str, outline: &mut FileOutline) {
    let def_re = Regex::new(r"^(\s*)(async\s+)?def\s+(\w+)\s*\([^)]*\)(\s*->\s*[^:]+)?").unwrap();
    let class_re = Regex::new(r"^(\s*)class\s+(\w+)(\([^)]*\))?").unwrap();

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1;

        if let Some(caps) = def_re.captures(line) {
            let indent = caps.get(1).map_or(0, |m| m.as_str().len());
            let sig = caps.get(0).unwrap().as_str().trim_end_matches(':').trim();
            outline.push(OutlineItem::new(
                line_num,
                sig.to_string(),
                indent / 4,
                ItemKind::Function,
            ));
        } else if let Some(caps) = class_re.captures(line) {
            let indent = caps.get(1).map_or(0, |m| m.as_str().len());
            let sig = caps.get(0).unwrap().as_str().trim_end_matches(':').trim();
            outline.push(OutlineItem::new(
                line_num,
                sig.to_string(),
                indent / 4,
                ItemKind::Class,
            ));
        }
    }
}

/// Extract JavaScript/TypeScript outline
fn extract_js_outline(content: &str, outline: &mut FileOutline) {
    let fn_re =
        Regex::new(r"^(\s*)(export\s+)?(async\s+)?function\s+(\w+)\s*(<[^>]+>)?\s*\([^)]*\)")
            .unwrap();
    let arrow_re =
        Regex::new(r"^(\s*)(export\s+)?(const|let|var)\s+(\w+)\s*=\s*(async\s+)?\([^)]*\)\s*=>")
            .unwrap();
    let class_re = Regex::new(r"^(\s*)(export\s+)?class\s+(\w+)(\s+extends\s+\w+)?").unwrap();
    let method_re = Regex::new(r"^(\s*)(async\s+)?(\w+)\s*\([^)]*\)\s*\{").unwrap();

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1;

        if let Some(caps) = fn_re.captures(line) {
            let indent = caps.get(1).map_or(0, |m| m.as_str().len());
            let sig = caps.get(0).unwrap().as_str().trim();
            outline.push(OutlineItem::new(
                line_num,
                sig.to_string(),
                indent / 2,
                ItemKind::Function,
            ));
        } else if let Some(caps) = arrow_re.captures(line) {
            let indent = caps.get(1).map_or(0, |m| m.as_str().len());
            let sig = caps.get(0).unwrap().as_str().trim_end_matches("=>").trim();
            outline.push(OutlineItem::new(
                line_num,
                sig.to_string(),
                indent / 2,
                ItemKind::Function,
            ));
        } else if let Some(caps) = class_re.captures(line) {
            let indent = caps.get(1).map_or(0, |m| m.as_str().len());
            let sig = caps.get(0).unwrap().as_str().trim();
            outline.push(OutlineItem::new(
                line_num,
                sig.to_string(),
                indent / 2,
                ItemKind::Class,
            ));
        } else if let Some(caps) = method_re.captures(line) {
            let indent = caps.get(1).map_or(0, |m| m.as_str().len());
            // Only include methods with some indent (inside class)
            if indent > 0 {
                let sig = caps.get(0).unwrap().as_str().trim_end_matches('{').trim();
                outline.push(OutlineItem::new(
                    line_num,
                    sig.to_string(),
                    indent / 2,
                    ItemKind::Function,
                ));
            }
        }
    }
}

/// Extract Ruby outline
fn extract_ruby_outline(content: &str, outline: &mut FileOutline) {
    let def_re = Regex::new(r"^(\s*)def\s+(\w+[?!=]?)(\([^)]*\))?").unwrap();
    let class_re = Regex::new(r"^(\s*)class\s+(\w+)(\s*<\s*\w+)?").unwrap();
    let module_re = Regex::new(r"^(\s*)module\s+(\w+)").unwrap();

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1;

        if let Some(caps) = def_re.captures(line) {
            let indent = caps.get(1).map_or(0, |m| m.as_str().len());
            let sig = caps.get(0).unwrap().as_str().trim();
            outline.push(OutlineItem::new(
                line_num,
                sig.to_string(),
                indent / 2,
                ItemKind::Function,
            ));
        } else if let Some(caps) = class_re.captures(line) {
            let indent = caps.get(1).map_or(0, |m| m.as_str().len());
            let sig = caps.get(0).unwrap().as_str().trim();
            outline.push(OutlineItem::new(
                line_num,
                sig.to_string(),
                indent / 2,
                ItemKind::Class,
            ));
        } else if let Some(caps) = module_re.captures(line) {
            let indent = caps.get(1).map_or(0, |m| m.as_str().len());
            let sig = caps.get(0).unwrap().as_str().trim();
            outline.push(OutlineItem::new(
                line_num,
                sig.to_string(),
                indent / 2,
                ItemKind::Module,
            ));
        }
    }
}

/// Extract Go outline
fn extract_go_outline(content: &str, outline: &mut FileOutline) {
    let func_re =
        Regex::new(r"^func\s+(\([^)]+\)\s+)?(\w+)\s*\([^)]*\)(\s*\([^)]*\)|\s*\w+)?").unwrap();
    let type_struct_re = Regex::new(r"^type\s+(\w+)\s+struct").unwrap();
    let type_interface_re = Regex::new(r"^type\s+(\w+)\s+interface").unwrap();

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1;

        if let Some(caps) = func_re.captures(line) {
            let sig = caps.get(0).unwrap().as_str().trim();
            outline.push(OutlineItem::new(
                line_num,
                sig.to_string(),
                0,
                ItemKind::Function,
            ));
        } else if let Some(caps) = type_struct_re.captures(line) {
            let sig = caps.get(0).unwrap().as_str().trim();
            outline.push(OutlineItem::new(
                line_num,
                sig.to_string(),
                0,
                ItemKind::Struct,
            ));
        } else if let Some(caps) = type_interface_re.captures(line) {
            let sig = caps.get(0).unwrap().as_str().trim();
            outline.push(OutlineItem::new(
                line_num,
                sig.to_string(),
                0,
                ItemKind::Trait,
            ));
        }
    }
}

/// Extract Markdown outline (headings)
fn extract_markdown_outline(content: &str, outline: &mut FileOutline) {
    let heading_re = Regex::new(r"^(#{1,6})\s+(.+)$").unwrap();

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1;

        if let Some(caps) = heading_re.captures(line) {
            let level = caps.get(1).unwrap().as_str().len() as u8;
            let text = caps.get(2).unwrap().as_str().to_string();
            outline.push(OutlineItem::new(
                line_num,
                text,
                (level - 1) as usize,
                ItemKind::Heading(level),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_function() {
        let content = "pub fn test(x: i32) -> String {";
        let outline = extract_outline(content, "test.rs");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("pub fn test"));
        assert_eq!(outline.items[0].kind, ItemKind::Function);
    }

    #[test]
    fn rust_async_function() {
        let content = "pub async fn fetch() -> Result<()> {";
        let outline = extract_outline(content, "test.rs");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("async fn fetch"));
    }

    #[test]
    fn rust_struct() {
        let content = "pub struct Config<T> {";
        let outline = extract_outline(content, "test.rs");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("struct Config"));
        assert_eq!(outline.items[0].kind, ItemKind::Struct);
    }

    #[test]
    fn rust_enum() {
        let content = "pub enum Status {";
        let outline = extract_outline(content, "test.rs");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("enum Status"));
        assert_eq!(outline.items[0].kind, ItemKind::Enum);
    }

    #[test]
    fn rust_trait() {
        let content = "pub trait Handler {";
        let outline = extract_outline(content, "test.rs");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("trait Handler"));
        assert_eq!(outline.items[0].kind, ItemKind::Trait);
    }

    #[test]
    fn rust_impl() {
        let content = "impl Config {";
        let outline = extract_outline(content, "test.rs");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("impl Config"));
        assert_eq!(outline.items[0].kind, ItemKind::Impl);
    }

    #[test]
    fn rust_impl_for() {
        let content = "impl Handler for Config {";
        let outline = extract_outline(content, "test.rs");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("impl Handler for Config"));
    }

    #[test]
    fn rust_mod() {
        let content = "pub mod utils;";
        let outline = extract_outline(content, "test.rs");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("mod utils"));
        assert_eq!(outline.items[0].kind, ItemKind::Module);
    }

    #[test]
    fn rust_const() {
        let content = "pub const MAX_SIZE: usize = 1024;";
        let outline = extract_outline(content, "test.rs");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("const MAX_SIZE"));
        assert_eq!(outline.items[0].kind, ItemKind::Const);
    }

    #[test]
    fn rust_type() {
        let content = "pub type Result<T> = std::result::Result<T, Error>;";
        let outline = extract_outline(content, "test.rs");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("type Result"));
        assert_eq!(outline.items[0].kind, ItemKind::Type);
    }

    #[test]
    fn rust_nested() {
        let content = r#"
impl Config {
    pub fn new() -> Self {
    }
}
"#;
        let outline = extract_outline(content, "test.rs");
        assert_eq!(outline.len(), 2);
        assert_eq!(outline.items[0].level, 0);
        assert_eq!(outline.items[1].level, 1);
    }

    #[test]
    fn python_function() {
        let content = "def process(data: list) -> dict:";
        let outline = extract_outline(content, "test.py");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("def process"));
        assert_eq!(outline.items[0].kind, ItemKind::Function);
    }

    #[test]
    fn python_async_function() {
        let content = "async def fetch(url):";
        let outline = extract_outline(content, "test.py");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("async def fetch"));
    }

    #[test]
    fn python_class() {
        let content = "class Handler(BaseHandler):";
        let outline = extract_outline(content, "test.py");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("class Handler"));
        assert_eq!(outline.items[0].kind, ItemKind::Class);
    }

    #[test]
    fn python_nested() {
        let content = r#"
class Handler:
    def process(self):
        pass
"#;
        let outline = extract_outline(content, "test.py");
        assert_eq!(outline.len(), 2);
        assert_eq!(outline.items[0].level, 0);
        assert_eq!(outline.items[1].level, 1);
    }

    #[test]
    fn js_function() {
        let content = "export async function fetchData(url) {";
        let outline = extract_outline(content, "test.js");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("function fetchData"));
        assert_eq!(outline.items[0].kind, ItemKind::Function);
    }

    #[test]
    fn js_arrow_function() {
        let content = "const handler = async (req, res) =>";
        let outline = extract_outline(content, "test.js");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("const handler"));
    }

    #[test]
    fn js_class() {
        let content = "export class UserService extends Service {";
        let outline = extract_outline(content, "test.js");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("class UserService"));
        assert_eq!(outline.items[0].kind, ItemKind::Class);
    }

    #[test]
    fn ts_function() {
        let content = "export function process<T>(data: T[]): T[] {";
        let outline = extract_outline(content, "test.ts");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("function process"));
    }

    #[test]
    fn ruby_def() {
        let content = "def process(data)";
        let outline = extract_outline(content, "test.rb");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("def process"));
        assert_eq!(outline.items[0].kind, ItemKind::Function);
    }

    #[test]
    fn ruby_predicate() {
        let content = "def valid?";
        let outline = extract_outline(content, "test.rb");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("def valid?"));
    }

    #[test]
    fn ruby_class() {
        let content = "class Handler < BaseHandler";
        let outline = extract_outline(content, "test.rb");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("class Handler"));
        assert_eq!(outline.items[0].kind, ItemKind::Class);
    }

    #[test]
    fn ruby_module() {
        let content = "module Utils";
        let outline = extract_outline(content, "test.rb");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("module Utils"));
        assert_eq!(outline.items[0].kind, ItemKind::Module);
    }

    #[test]
    fn go_func() {
        let content = "func (s *Server) Handle(w http.ResponseWriter, r *http.Request) {";
        let outline = extract_outline(content, "test.go");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("func"));
        assert!(outline.items[0].text.contains("Handle"));
        assert_eq!(outline.items[0].kind, ItemKind::Function);
    }

    #[test]
    fn go_struct() {
        let content = "type Config struct {";
        let outline = extract_outline(content, "test.go");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("type Config struct"));
        assert_eq!(outline.items[0].kind, ItemKind::Struct);
    }

    #[test]
    fn go_interface() {
        let content = "type Handler interface {";
        let outline = extract_outline(content, "test.go");
        assert_eq!(outline.len(), 1);
        assert!(outline.items[0].text.contains("type Handler interface"));
        assert_eq!(outline.items[0].kind, ItemKind::Trait);
    }

    #[test]
    fn markdown_headings() {
        let content = r#"
# Title
## Section 1
### Subsection
## Section 2
"#;
        let outline = extract_outline(content, "test.md");
        assert_eq!(outline.len(), 4);
        assert_eq!(outline.items[0].text, "Title");
        assert_eq!(outline.items[0].kind, ItemKind::Heading(1));
        assert_eq!(outline.items[0].level, 0);
        assert_eq!(outline.items[1].text, "Section 1");
        assert_eq!(outline.items[1].kind, ItemKind::Heading(2));
        assert_eq!(outline.items[1].level, 1);
        assert_eq!(outline.items[2].text, "Subsection");
        assert_eq!(outline.items[2].kind, ItemKind::Heading(3));
        assert_eq!(outline.items[2].level, 2);
    }

    #[test]
    fn unknown_extension() {
        let content = "some content";
        let outline = extract_outline(content, "test.xyz");
        assert!(outline.is_empty());
    }

    #[test]
    fn empty_content() {
        let outline = extract_outline("", "test.rs");
        assert!(outline.is_empty());
    }

    #[test]
    fn line_numbers_correct() {
        let content = r#"
pub fn first() {}
pub fn second() {}
"#;
        let outline = extract_outline(content, "test.rs");
        assert_eq!(outline.len(), 2);
        assert_eq!(outline.items[0].line, 2);
        assert_eq!(outline.items[1].line, 3);
    }
}
