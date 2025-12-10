//! Code completion functionality for Vela LSP

use lsp_types::{CompletionItem, CompletionItemKind, CompletionList, CompletionParams};

/// Completion provider for Vela language
pub struct CompletionProvider {
    /// Cache of completion items for performance
    keyword_cache: Vec<CompletionItem>,
    type_cache: Vec<CompletionItem>,
    function_cache: Vec<CompletionItem>,
}

impl CompletionProvider {
    /// Create a new completion provider
    pub fn new() -> Self {
        Self {
            keyword_cache: Self::build_keyword_completions(),
            type_cache: Self::build_type_completions(),
            function_cache: Self::build_function_completions(),
        }
    }

    /// Get completions for the given parameters
    pub fn get_completions(&self, params: &CompletionParams) -> CompletionList {
        // For now, return all available completions
        // In the future, this could be context-aware based on the document and position
        let mut items = Vec::new();
        items.extend(self.keyword_cache.clone());
        items.extend(self.type_cache.clone());
        items.extend(self.function_cache.clone());

        CompletionList {
            is_incomplete: false,
            items,
        }
    }

    /// Build keyword completion items
    fn build_keyword_completions() -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "fn".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Function declaration".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(lsp_types::MarkupContent {
                    kind: lsp_types::MarkupKind::Markdown,
                    value: "Declare a function with parameters and return type.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "class".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Class declaration".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(lsp_types::MarkupContent {
                    kind: lsp_types::MarkupKind::Markdown,
                    value: "Declare a class with methods and properties.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "if".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Conditional statement".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "else".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Alternative conditional branch".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "match".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Pattern matching".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "state".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Reactive state variable".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "import".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Import statement".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "public".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Public visibility modifier".to_string()),
                ..Default::default()
            },
        ]
    }

    /// Build type completion items
    fn build_type_completions() -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "Number".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("64-bit integer type".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(lsp_types::MarkupContent {
                    kind: lsp_types::MarkupKind::Markdown,
                    value: "64-bit signed integer type for whole numbers.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "String".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("UTF-8 string type".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(lsp_types::MarkupContent {
                    kind: lsp_types::MarkupKind::Markdown,
                    value: "UTF-8 encoded string type.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "Bool".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("Boolean type".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "Float".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("64-bit floating point type".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "List".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("Dynamic array type".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "Option".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("Optional value type".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "Result".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("Result type for error handling".to_string()),
                ..Default::default()
            },
        ]
    }

    /// Build function completion items
    fn build_function_completions() -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "print".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("fn print(value: any) -> void".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(lsp_types::MarkupContent {
                    kind: lsp_types::MarkupKind::Markdown,
                    value: "Print a value to the console.".to_string(),
                })),
                insert_text: Some("print(${1:value})".to_string()),
                insert_text_format: Some(lsp_types::InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "println".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("fn println(value: any) -> void".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(lsp_types::MarkupContent {
                    kind: lsp_types::MarkupKind::Markdown,
                    value: "Print a value to the console with a newline.".to_string(),
                })),
                insert_text: Some("println(${1:value})".to_string()),
                insert_text_format: Some(lsp_types::InsertTextFormat::SNIPPET),
                ..Default::default()
            },
        ]
    }
}

impl Default for CompletionProvider {
    fn default() -> Self {
        Self::new()
    }
}