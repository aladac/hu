/// An item in a file outline (function, struct, class, heading, etc.)
#[derive(Debug, Clone, PartialEq)]
pub struct OutlineItem {
    /// Line number where this item starts (1-indexed)
    pub line: usize,
    /// The signature or heading text
    pub text: String,
    /// Indent level (0 for top-level)
    pub level: usize,
    /// Kind of item (function, struct, class, heading, etc.)
    pub kind: ItemKind,
}

impl OutlineItem {
    pub fn new(line: usize, text: String, level: usize, kind: ItemKind) -> Self {
        Self {
            line,
            text,
            level,
            kind,
        }
    }
}

/// Kind of outline item
#[derive(Debug, Clone, PartialEq)]
pub enum ItemKind {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Class,
    Module,
    Const,
    Type,
    Heading(u8), // Heading level (1-6)
    Other,
}

impl ItemKind {
    pub fn icon(&self) -> &'static str {
        match self {
            ItemKind::Function => "fn",
            ItemKind::Struct => "struct",
            ItemKind::Enum => "enum",
            ItemKind::Trait => "trait",
            ItemKind::Impl => "impl",
            ItemKind::Class => "class",
            ItemKind::Module => "mod",
            ItemKind::Const => "const",
            ItemKind::Type => "type",
            ItemKind::Heading(n) => match n {
                1 => "#",
                2 => "##",
                3 => "###",
                _ => "####",
            },
            ItemKind::Other => "",
        }
    }
}

/// File outline (collection of items)
#[derive(Debug, Clone, Default)]
pub struct FileOutline {
    pub items: Vec<OutlineItem>,
}

impl FileOutline {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn push(&mut self, item: OutlineItem) {
        self.items.push(item);
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

/// Result of reading a file with options
#[cfg(test)]
#[derive(Debug, Clone)]
pub enum ReadOutput {
    /// Full file content
    Full(String),
    /// File outline (signatures)
    Outline(FileOutline),
    /// Public interface only
    Interface(Vec<OutlineItem>),
    /// Lines around a specific line
    Around {
        lines: Vec<(usize, String)>,
        center: usize,
        total_lines: usize,
    },
    /// Git diff output
    Diff(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outline_item_new() {
        let item = OutlineItem::new(10, "fn test()".to_string(), 0, ItemKind::Function);
        assert_eq!(item.line, 10);
        assert_eq!(item.text, "fn test()");
        assert_eq!(item.level, 0);
        assert_eq!(item.kind, ItemKind::Function);
    }

    #[test]
    fn outline_item_clone() {
        let item = OutlineItem::new(1, "test".to_string(), 0, ItemKind::Function);
        let cloned = item.clone();
        assert_eq!(item, cloned);
    }

    #[test]
    fn outline_item_debug() {
        let item = OutlineItem::new(1, "test".to_string(), 0, ItemKind::Function);
        let debug = format!("{:?}", item);
        assert!(debug.contains("OutlineItem"));
    }

    #[test]
    fn item_kind_icon_function() {
        assert_eq!(ItemKind::Function.icon(), "fn");
    }

    #[test]
    fn item_kind_icon_struct() {
        assert_eq!(ItemKind::Struct.icon(), "struct");
    }

    #[test]
    fn item_kind_icon_enum() {
        assert_eq!(ItemKind::Enum.icon(), "enum");
    }

    #[test]
    fn item_kind_icon_trait() {
        assert_eq!(ItemKind::Trait.icon(), "trait");
    }

    #[test]
    fn item_kind_icon_impl() {
        assert_eq!(ItemKind::Impl.icon(), "impl");
    }

    #[test]
    fn item_kind_icon_class() {
        assert_eq!(ItemKind::Class.icon(), "class");
    }

    #[test]
    fn item_kind_icon_module() {
        assert_eq!(ItemKind::Module.icon(), "mod");
    }

    #[test]
    fn item_kind_icon_const() {
        assert_eq!(ItemKind::Const.icon(), "const");
    }

    #[test]
    fn item_kind_icon_type() {
        assert_eq!(ItemKind::Type.icon(), "type");
    }

    #[test]
    fn item_kind_icon_other() {
        assert_eq!(ItemKind::Other.icon(), "");
    }

    #[test]
    fn item_kind_icon_heading() {
        assert_eq!(ItemKind::Heading(1).icon(), "#");
        assert_eq!(ItemKind::Heading(2).icon(), "##");
        assert_eq!(ItemKind::Heading(3).icon(), "###");
        assert_eq!(ItemKind::Heading(4).icon(), "####");
        assert_eq!(ItemKind::Heading(5).icon(), "####");
        assert_eq!(ItemKind::Heading(6).icon(), "####");
    }

    #[test]
    fn item_kind_clone() {
        let kind = ItemKind::Function;
        let cloned = kind.clone();
        assert_eq!(kind, cloned);
    }

    #[test]
    fn item_kind_debug() {
        let kind = ItemKind::Struct;
        let debug = format!("{:?}", kind);
        assert!(debug.contains("Struct"));
    }

    #[test]
    fn file_outline_new() {
        let outline = FileOutline::new();
        assert!(outline.is_empty());
        assert_eq!(outline.len(), 0);
    }

    #[test]
    fn file_outline_default() {
        let outline = FileOutline::default();
        assert!(outline.is_empty());
    }

    #[test]
    fn file_outline_push() {
        let mut outline = FileOutline::new();
        outline.push(OutlineItem::new(
            1,
            "test".to_string(),
            0,
            ItemKind::Function,
        ));
        assert_eq!(outline.len(), 1);
        assert!(!outline.is_empty());
    }

    #[test]
    fn file_outline_clone() {
        let mut outline = FileOutline::new();
        outline.push(OutlineItem::new(
            1,
            "test".to_string(),
            0,
            ItemKind::Function,
        ));
        let cloned = outline.clone();
        assert_eq!(outline.len(), cloned.len());
    }

    #[test]
    fn file_outline_debug() {
        let outline = FileOutline::new();
        let debug = format!("{:?}", outline);
        assert!(debug.contains("FileOutline"));
    }

    #[test]
    fn read_output_full() {
        let output = ReadOutput::Full("content".to_string());
        assert!(matches!(output, ReadOutput::Full(_)));
    }

    #[test]
    fn read_output_outline() {
        let output = ReadOutput::Outline(FileOutline::new());
        assert!(matches!(output, ReadOutput::Outline(_)));
    }

    #[test]
    fn read_output_interface() {
        let output = ReadOutput::Interface(vec![]);
        assert!(matches!(output, ReadOutput::Interface(_)));
    }

    #[test]
    fn read_output_around() {
        let output = ReadOutput::Around {
            lines: vec![(1, "line".to_string())],
            center: 1,
            total_lines: 10,
        };
        assert!(matches!(output, ReadOutput::Around { .. }));
    }

    #[test]
    fn read_output_diff() {
        let output = ReadOutput::Diff("diff output".to_string());
        assert!(matches!(output, ReadOutput::Diff(_)));
    }

    #[test]
    fn read_output_clone() {
        let output = ReadOutput::Full("test".to_string());
        let cloned = output.clone();
        if let (ReadOutput::Full(a), ReadOutput::Full(b)) = (output, cloned) {
            assert_eq!(a, b);
        } else {
            panic!("Expected Full");
        }
    }

    #[test]
    fn read_output_debug() {
        let output = ReadOutput::Diff("test".to_string());
        let debug = format!("{:?}", output);
        assert!(debug.contains("Diff"));
    }
}
