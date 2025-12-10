//! Language server implementation

use anyhow::Result;
use lsp_server::{Connection, Message, Request, Response};
use lsp_types::{
    InitializeParams, InitializeResult, ServerCapabilities, ServerInfo,
    TextDocumentSyncCapability, TextDocumentSyncKind, CompletionParams,
    CompletionList, CompletionItem, CompletionItemKind, Position,
    CompletionOptions, HoverProviderCapability, HoverParams, Hover,
    MarkupContent, MarkupKind, GotoDefinitionParams,
    GotoDefinitionResponse, Location, Range, SignatureHelpParams,
    SignatureHelp, SignatureInformation, ParameterInformation,
    SignatureHelpOptions, ReferenceParams, Url,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

/// Document store for managing open text documents
#[derive(Debug, Default)]
pub struct DocumentStore {
    documents: HashMap<lsp_types::Url, String>,
}

/// Context information for a function call
#[derive(Debug)]
struct FunctionCallContext {
    function_name: String,
    active_parameter: usize,
}

impl DocumentStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_document(&mut self, uri: lsp_types::Url, content: String) {
        self.documents.insert(uri, content);
    }

    pub fn get_document(&self, uri: &lsp_types::Url) -> Option<&String> {
        self.documents.get(uri)
    }

    pub fn remove_document(&mut self, uri: &lsp_types::Url) {
        self.documents.remove(uri);
    }
}

/// Main language server struct
pub struct LanguageServer {
    connection: Connection,
    document_store: Arc<Mutex<DocumentStore>>,
}

impl LanguageServer {
    /// Create a new language server instance
    pub fn new() -> Result<Self> {
        let (connection, io_threads) = Connection::stdio();

        // Spawn the IO threads in the background
        std::thread::spawn(move || {
            io_threads.join().expect("IO threads panicked");
        });

        Ok(Self {
            connection,
            document_store: Arc::new(Mutex::new(DocumentStore::new())),
        })
    }

    /// Run the language server main loop
    pub fn run(mut self) -> Result<()> {
        info!("Vela Language Server starting...");

        loop {
            let message = match self.connection.receiver.recv() {
                Ok(msg) => msg,
                Err(err) => {
                    error!("Error receiving message: {}", err);
                    break;
                }
            };

            match message {
                Message::Request(request) => {
                    if let Err(err) = self.handle_request(request) {
                        error!("Error handling request: {}", err);
                    }
                }
                Message::Response(response) => {
                    info!("Received response: {:?}", response);
                }
                Message::Notification(notification) => {
                    if let Err(err) = self.handle_notification(notification) {
                        error!("Error handling notification: {}", err);
                    }
                }
            }
        }

        info!("Vela Language Server shutting down");
        Ok(())
    }

    /// Handle LSP requests
    fn handle_request(&mut self, request: Request) -> Result<()> {
        let response = match request.method.as_str() {
            "initialize" => self.handle_initialize(request)?,
            "shutdown" => self.handle_shutdown(request)?,
            "textDocument/completion" => self.handle_completion(request)?,
            "textDocument/hover" => self.handle_hover(request)?,
            "textDocument/definition" => self.handle_definition(request)?,
            "textDocument/signatureHelp" => self.handle_signature_help(request)?,
            "textDocument/references" => self.handle_references(request)?,
            _ => {
                warn!("Unhandled request method: {}", request.method);
                Response::new_err(
                    request.id,
                    lsp_server::ErrorCode::MethodNotFound as i32,
                    format!("Method '{}' not implemented", request.method),
                )
            }
        };

        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    /// Handle LSP notifications
    fn handle_notification(&mut self, notification: lsp_server::Notification) -> Result<()> {
        match notification.method.as_str() {
            "initialized" => {
                info!("Client initialized successfully");
            }
            "textDocument/didOpen" => {
                self.handle_did_open(notification)?;
            }
            "textDocument/didChange" => {
                self.handle_did_change(notification)?;
            }
            "textDocument/didClose" => {
                self.handle_did_close(notification)?;
            }
            "exit" => {
                info!("Received exit notification, shutting down");
                return Ok(()); // This will break the main loop
            }
            _ => {
                info!("Received notification: {}", notification.method);
            }
        }
        Ok(())
    }

    /// Handle initialize request
    fn handle_initialize(&self, request: Request) -> Result<Response> {
        let params: InitializeParams = serde_json::from_value(request.params)
            .map_err(|e| anyhow::anyhow!("Invalid initialize params: {}", e))?;

        info!("Initializing server for client: {:?}", params.client_info);

        let capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::FULL,
            )),
            completion_provider: Some(CompletionOptions {
                resolve_provider: Some(false),
                trigger_characters: Some(vec![".".to_string()]),
                ..Default::default()
            }),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            definition_provider: Some(lsp_types::OneOf::Left(true)),
            signature_help_provider: Some(SignatureHelpOptions {
                trigger_characters: Some(vec!["(".to_string()]),
                retrigger_characters: Some(vec![",".to_string()]),
                ..Default::default()
            }),
            references_provider: Some(lsp_types::OneOf::Left(true)),
            // TODO: Add more capabilities as we implement them
            ..Default::default()
        };

        let server_info = ServerInfo {
            name: "Vela Language Server".to_string(),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        };

        let result = InitializeResult {
            capabilities,
            server_info: Some(server_info),
        };

        let response = Response::new_ok(request.id, result);
        Ok(response)
    }

    /// Handle shutdown request
    fn handle_shutdown(&self, request: Request) -> Result<Response> {
        info!("Shutdown requested");
        let response = Response::new_ok(request.id, ());
        Ok(response)
    }

    /// Handle textDocument/completion request
    fn handle_completion(&self, request: Request) -> Result<Response> {
        let params: CompletionParams = serde_json::from_value(request.params)
            .map_err(|e| anyhow::anyhow!("Invalid completion params: {}", e))?;

        info!("Completion requested at position: {:?}", params.text_document_position.position);

        let completions = self.compute_completions(&params)?;

        let response = Response::new_ok(request.id, completions);
        Ok(response)
    }

    /// Handle textDocument/hover request
    fn handle_hover(&self, request: Request) -> Result<Response> {
        let params: HoverParams = serde_json::from_value(request.params)
            .map_err(|e| anyhow::anyhow!("Invalid hover params: {}", e))?;

        info!("Hover requested at position: {:?}", params.text_document_position_params.position);

        let hover = self.compute_hover(&params)?;

        let response = Response::new_ok(request.id, hover);
        Ok(response)
    }

    /// Handle textDocument/definition request
    fn handle_definition(&self, request: Request) -> Result<Response> {
        let params: GotoDefinitionParams = serde_json::from_value(request.params)
            .map_err(|e| anyhow::anyhow!("Invalid definition params: {}", e))?;

        info!("Definition requested at position: {:?}", params.text_document_position_params.position);

        let definition = self.compute_definition(&params)?;

        let response = Response::new_ok(request.id, definition);
        Ok(response)
    }

    /// Handle textDocument/signatureHelp request
    fn handle_signature_help(&self, request: Request) -> Result<Response> {
        let params: SignatureHelpParams = serde_json::from_value(request.params)
            .map_err(|e| anyhow::anyhow!("Invalid signatureHelp params: {}", e))?;

        info!("Signature help requested at position: {:?}", params.text_document_position_params.position);

        let signature_help = self.compute_signature_help(&params)?;

        let response = Response::new_ok(request.id, signature_help);
        Ok(response)
    }

    /// Handle textDocument/references request
    fn handle_references(&self, request: Request) -> Result<Response> {
        let params: ReferenceParams = serde_json::from_value(request.params)
            .map_err(|e| anyhow::anyhow!("Invalid references params: {}", e))?;

        info!("References requested at position: {:?}", params.text_document_position.position);

        let references = self.compute_references(&params)?;

        let response = Response::new_ok(request.id, references);
        Ok(response)
    }

    /// Handle textDocument/didOpen
    fn handle_did_open(&self, notification: lsp_server::Notification) -> Result<()> {
        let params: lsp_types::DidOpenTextDocumentParams =
            serde_json::from_value(notification.params)
                .map_err(|e| anyhow::anyhow!("Invalid didOpen params: {}", e))?;

        let uri = params.text_document.uri.clone();
        let mut store = self.document_store.lock().unwrap();
        store.update_document(params.text_document.uri, params.text_document.text);

        info!("Opened document: {}", uri);
        Ok(())
    }

    /// Handle textDocument/didChange
    fn handle_did_change(&self, notification: lsp_server::Notification) -> Result<()> {
        let params: lsp_types::DidChangeTextDocumentParams =
            serde_json::from_value(notification.params)
                .map_err(|e| anyhow::anyhow!("Invalid didChange params: {}", e))?;

        // For now, we only handle full content changes
        if let Some(change) = params.content_changes.first() {
            let mut store = self.document_store.lock().unwrap();
            store.update_document(params.text_document.uri.clone(), change.text.clone());

            info!("Updated document: {}", params.text_document.uri);
        }

        Ok(())
    }

    /// Handle textDocument/didClose
    fn handle_did_close(&self, notification: lsp_server::Notification) -> Result<()> {
        let params: lsp_types::DidCloseTextDocumentParams =
            serde_json::from_value(notification.params)
                .map_err(|e| anyhow::anyhow!("Invalid didClose params: {}", e))?;

        let uri = params.text_document.uri.clone();
        let mut store = self.document_store.lock().unwrap();
        store.remove_document(&params.text_document.uri);

        info!("Closed document: {}", uri);
        Ok(())
    }
}

/// Completion context for determining what kind of completions to provide
enum CompletionContext {
    Keyword,
    Type,
    Function,
    Variable,
    Unknown,
}

impl LanguageServer {
    /// Compute completion items based on the current context
    fn compute_completions(&self, params: &CompletionParams) -> Result<CompletionList> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        // Get document content
        let store = self.document_store.lock().unwrap();
        let document = match store.get_document(uri) {
            Some(doc) => doc,
            None => {
                // Return empty completion list if document not found
                return Ok(CompletionList {
                    is_incomplete: false,
                    items: vec![],
                });
            }
        };

        // Analyze context at position
        let context = self.analyze_completion_context(document, position);

        // Generate completion items based on context
        let items = match context {
            CompletionContext::Keyword => self.keyword_completions(),
            CompletionContext::Type => self.type_completions(),
            CompletionContext::Function => self.function_completions(),
            CompletionContext::Variable => self.variable_completions(),
            CompletionContext::Unknown => self.basic_completions(),
        };

        Ok(CompletionList {
            is_incomplete: false,
            items,
        })
    }

    /// Compute hover information based on the current position
    fn compute_hover(&self, params: &HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Get document content
        let store = self.document_store.lock().unwrap();
        let document = match store.get_document(uri) {
            Some(doc) => doc,
            None => return Ok(None), // No document found
        };

        // Analyze the symbol at position
        let hover_info = self.analyze_hover_symbol(document, position);

        Ok(hover_info)
    }

    /// Compute definition location based on the current position
    fn compute_definition(&self, params: &GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Get document content
        let store = self.document_store.lock().unwrap();
        let document = match store.get_document(uri) {
            Some(doc) => doc,
            None => return Ok(None), // No document found
        };

        // Analyze the symbol at position and find its definition
        let definition_location = self.analyze_definition_symbol(document, position, uri);

        Ok(definition_location.map(GotoDefinitionResponse::Scalar))
    }

    /// Compute signature help based on the current position
    fn compute_signature_help(&self, params: &SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Get document content
        let store = self.document_store.lock().unwrap();
        let document = match store.get_document(uri) {
            Some(doc) => doc,
            None => return Ok(None), // No document found
        };

        // Analyze the function call at position
        let signature_help = self.analyze_signature_help(document, position);

        Ok(signature_help)
    }

    /// Compute references for the symbol at the given position
    fn compute_references(&self, params: &ReferenceParams) -> Result<Vec<Location>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        // Get document content
        let store = self.document_store.lock().unwrap();
        let document = match store.get_document(uri) {
            Some(doc) => doc,
            None => return Ok(vec![]), // No document found
        };

        // Find the symbol at the position
        let symbol = self.analyze_references_symbol(document, position)?;
        if symbol.is_empty() {
            return Ok(vec![]);
        }

        // Find all references to this symbol in the document
        let references = self.find_symbol_references(document, &symbol, uri.clone());

        Ok(references)
    }

    /// Analyze the completion context at the given position
    fn analyze_completion_context(&self, document: &str, position: Position) -> CompletionContext {
        // Simple context analysis - this could be much more sophisticated
        // For now, we just provide basic keyword completions

        // Convert position to byte offset
        let lines: Vec<&str> = document.lines().collect();
        if position.line as usize >= lines.len() {
            return CompletionContext::Unknown;
        }

        let line = lines[position.line as usize];
        let char_pos = position.character as usize;

        if char_pos > line.len() {
            return CompletionContext::Unknown;
        }

        let before_cursor = &line[..char_pos];

        // Very basic heuristics
        if before_cursor.ends_with("fn ") || before_cursor.trim().is_empty() {
            CompletionContext::Function
        } else if before_cursor.ends_with("let ") || before_cursor.ends_with("state ") {
            CompletionContext::Variable
        } else if before_cursor.ends_with(": ") {
            CompletionContext::Type
        } else {
            CompletionContext::Keyword
        }
    }

    /// Analyze the symbol at the given position for hover information
    fn analyze_hover_symbol(&self, document: &str, position: Position) -> Option<Hover> {
        // Convert position to byte offset
        let lines: Vec<&str> = document.lines().collect();
        if position.line as usize >= lines.len() {
            return None;
        }

        let line = lines[position.line as usize];
        let char_pos = position.character as usize;

        if char_pos > line.len() {
            return None;
        }

        // Extract word at position (simple word boundary detection)
        let word = self.extract_word_at_position(line, char_pos)?;

        // Generate hover information based on the word
        self.generate_hover_for_word(&word)
    }

    /// Extract word at the given character position in a line
    fn extract_word_at_position(&self, line: &str, char_pos: usize) -> Option<String> {
        let chars: Vec<char> = line.chars().collect();
        if char_pos >= chars.len() {
            return None;
        }

        // Find word boundaries (simple: alphanumeric and underscore)
        let mut start = char_pos;
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }

        let mut end = char_pos;
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }

        if start < end {
            Some(chars[start..end].iter().collect())
        } else {
            None
        }
    }

    /// Generate hover information for a given word
    fn generate_hover_for_word(&self, word: &str) -> Option<Hover> {
        let (content, range) = match word {
            // Keywords
            "fn" => (
                "**fn** - Function declaration\n\nDeclare a function with parameters and return type.\n\n```vela\nfn add(a: Number, b: Number) -> Number {\n  return a + b\n}\n```",
                None,
            ),
            "let" => (
                "**let** - Variable declaration (deprecated)\n\nNote: Variables are immutable by default in Vela. Use `state` for mutable reactive variables.",
                None,
            ),
            "state" => (
                "**state** - Reactive state variable\n\nDeclare a mutable variable that triggers reactivity.\n\n```vela\nstate count: Number = 0\ncount = count + 1  // Triggers reactivity\n```",
                None,
            ),
            "if" => (
                "**if** - Conditional statement\n\nExecute code conditionally.\n\n```vela\nif age >= 18 {\n  \"adult\"\n} else {\n  \"minor\"\n}\n```",
                None,
            ),
            "match" => (
                "**match** - Pattern matching\n\nExhaustive pattern matching expression.\n\n```vela\nmatch value {\n  1 => \"one\"\n  2 => \"two\"\n  _ => \"other\"\n}\n```",
                None,
            ),
            "class" => (
                "**class** - Class declaration\n\nDefine a class with methods and properties.\n\n```vela\nclass Person {\n  constructor(name: String) {\n    this.name = name\n  }\n}\n```",
                None,
            ),
            "interface" => (
                "**interface** - Interface declaration\n\nDefine a contract for types.\n\n```vela\ninterface Drawable {\n  fn draw() -> void\n}\n```",
                None,
            ),
            "public" => (
                "**public** - Public modifier\n\nMake declarations accessible from other modules.",
                None,
            ),
            "return" => (
                "**return** - Return statement\n\nReturn a value from a function.",
                None,
            ),

            // Types
            "String" => (
                "**String** - Text string type\n\nRepresents textual data.\n\n```vela\nname: String = \"Vela\"\nmessage: String = \"Hello, ${name}!\"\n```",
                None,
            ),
            "Number" => (
                "**Number** - Integer type\n\n64-bit signed integer.\n\n```vela\nage: Number = 37\ncount: Number = 0\n```",
                None,
            ),
            "Float" => (
                "**Float** - Floating point type\n\n64-bit floating point number.\n\n```vela\nprice: Float = 19.99\npi: Float = 3.14159\n```",
                None,
            ),
            "Bool" => (
                "**Bool** - Boolean type\n\nTrue or false values.\n\n```vela\nisActive: Bool = true\nhasPermission: Bool = false\n```",
                None,
            ),
            "void" => (
                "**void** - No return type\n\nIndicates a function returns nothing.",
                None,
            ),

            // Functions
            "print" => (
                "**print** - Print to console\n\nPrints a value to the console.\n\n```vela\nprint(\"Hello, World!\")\nprint(42)\n```",
                None,
            ),
            "len" => (
                "**len** - Get collection length\n\nReturns the length of a collection.\n\n```vela\nnumbers = [1, 2, 3]\nlength = len(numbers)  // 3\n```",
                None,
            ),

            // Unknown words
            _ => return None,
        };

        Some(Hover {
            contents: lsp_types::HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content.to_string(),
            }),
            range,
        })
    }

    /// Analyze the symbol at the given position and find its definition location
    fn analyze_definition_symbol(&self, document: &str, position: Position, uri: &lsp_types::Url) -> Option<Location> {
        // Convert position to byte offset
        let lines: Vec<&str> = document.lines().collect();
        if position.line as usize >= lines.len() {
            return None;
        }

        let line = lines[position.line as usize];
        let char_pos = position.character as usize;

        if char_pos > line.len() {
            return None;
        }

        // Extract word at position
        let word = self.extract_word_at_position(line, char_pos)?;

        // For built-in symbols, we can't provide definitions
        // For user-defined symbols, search for their definition in the document
        self.find_symbol_definition(document, &word, uri)
    }

    /// Find the definition location of a symbol in the document
    fn find_symbol_definition(&self, document: &str, symbol: &str, uri: &lsp_types::Url) -> Option<Location> {
        let lines: Vec<&str> = document.lines().collect();

        // Vela-specific pattern matching for symbol definitions

        // Function definitions: fn symbol_name(...) -> ...
        let fn_pattern = format!("fn {}", symbol);

        // State variable declarations: state symbol_name: Type = ...
        let state_pattern = format!("state {}:", symbol);

        // Class definitions: class ClassName ...
        let class_pattern = format!("class {}", symbol);

        // Interface definitions: interface InterfaceName ...
        let interface_pattern = format!("interface {}", symbol);

        // Enum definitions: enum EnumName ...
        let enum_pattern = format!("enum {}", symbol);

        // Type aliases: type TypeName = ...
        let type_pattern = format!("type {} =", symbol);

        // Variable declarations: symbol_name: Type = ...
        // This is trickier as we need to find lines where symbol appears before :
        let var_pattern = format!("{}:", symbol);

        for (line_idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Check for function definition
            if trimmed.starts_with(&fn_pattern) && (trimmed.len() == fn_pattern.len() || trimmed.chars().nth(fn_pattern.len()).unwrap_or(' ').is_whitespace() || trimmed.chars().nth(fn_pattern.len()).unwrap_or(' ') == '(') {
                let char_start = line.find(&fn_pattern).unwrap_or(0);
                let char_end = char_start + fn_pattern.len();

                return Some(Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position { line: line_idx as u32, character: char_start as u32 },
                        end: Position { line: line_idx as u32, character: char_end as u32 },
                    },
                });
            }

            // Check for state variable declarations
            if trimmed.starts_with(&state_pattern) {
                let char_start = line.find(&state_pattern).unwrap_or(0);
                let char_end = char_start + state_pattern.len();

                return Some(Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position { line: line_idx as u32, character: char_start as u32 },
                        end: Position { line: line_idx as u32, character: char_end as u32 },
                    },
                });
            }

            // Check for class definition
            if trimmed.starts_with(&class_pattern) && (trimmed.len() == class_pattern.len() || trimmed.chars().nth(class_pattern.len()).unwrap_or(' ').is_whitespace()) {
                let char_start = line.find(&class_pattern).unwrap_or(0);
                let char_end = char_start + class_pattern.len();

                return Some(Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position { line: line_idx as u32, character: char_start as u32 },
                        end: Position { line: line_idx as u32, character: char_end as u32 },
                    },
                });
            }

            // Check for interface definition
            if trimmed.starts_with(&interface_pattern) && (trimmed.len() == interface_pattern.len() || trimmed.chars().nth(interface_pattern.len()).unwrap_or(' ').is_whitespace()) {
                let char_start = line.find(&interface_pattern).unwrap_or(0);
                let char_end = char_start + interface_pattern.len();

                return Some(Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position { line: line_idx as u32, character: char_start as u32 },
                        end: Position { line: line_idx as u32, character: char_end as u32 },
                    },
                });
            }

            // Check for enum definition
            if trimmed.starts_with(&enum_pattern) && (trimmed.len() == enum_pattern.len() || trimmed.chars().nth(enum_pattern.len()).unwrap_or(' ').is_whitespace()) {
                let char_start = line.find(&enum_pattern).unwrap_or(0);
                let char_end = char_start + enum_pattern.len();

                return Some(Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position { line: line_idx as u32, character: char_start as u32 },
                        end: Position { line: line_idx as u32, character: char_end as u32 },
                    },
                });
            }

            // Check for type alias
            if trimmed.starts_with(&type_pattern) {
                let char_start = line.find(&type_pattern).unwrap_or(0);
                let char_end = char_start + type_pattern.len();

                return Some(Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position { line: line_idx as u32, character: char_start as u32 },
                        end: Position { line: line_idx as u32, character: char_end as u32 },
                    },
                });
            }

            // Check for variable declarations: symbol: Type = ...
            // Look for symbol followed by : (but not part of other constructs)
            if let Some(colon_pos) = line.find(':') {
                let before_colon = &line[..colon_pos];
                if before_colon.trim() == symbol && !before_colon.contains("fn ") && !before_colon.contains("class ") && !before_colon.contains("interface ") && !before_colon.contains("enum ") && !before_colon.contains("type ") && !before_colon.contains("state ") {
                    let char_start = line.find(symbol).unwrap_or(0);
                    let char_end = char_start + symbol.len();

                    return Some(Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position { line: line_idx as u32, character: char_start as u32 },
                            end: Position { line: line_idx as u32, character: char_end as u32 },
                        },
                    });
                }
            }
        }

        // Symbol not found in document
        None
    }

    /// Analyze the function call at the given position for signature help
    fn analyze_signature_help(&self, document: &str, position: Position) -> Option<SignatureHelp> {
        // Convert position to byte offset
        let lines: Vec<&str> = document.lines().collect();
        if position.line as usize >= lines.len() {
            return None;
        }

        let line = lines[position.line as usize];
        let char_pos = position.character as usize;

        if char_pos > line.len() {
            return None;
        }

        // Find the function call context around the position
        let function_call = self.extract_function_call_context(line, char_pos)?;

        // Get signature information for the function
        let signatures = self.get_function_signatures(&function_call.function_name)?;

        // Determine active parameter based on position in call
        let active_parameter = self.calculate_active_parameter(&function_call, char_pos);

        Some(SignatureHelp {
            signatures,
            active_signature: Some(0), // We only provide one signature for now
            active_parameter: Some(active_parameter),
        })
    }

    /// Extract function call context from line at position
    fn extract_function_call_context(&self, line: &str, char_pos: usize) -> Option<FunctionCallContext> {
        // Find the opening parenthesis before the cursor
        let before_cursor = &line[..char_pos];
        let open_paren_pos = before_cursor.rfind('(')?;

        // Find the function name before the opening parenthesis
        let before_paren = &before_cursor[..open_paren_pos];
        let function_name = self.extract_word_at_position(before_paren, before_paren.len())?;

        // Count commas to determine active parameter
        let after_open = &line[open_paren_pos..char_pos];
        let comma_count = after_open.chars().filter(|&c| c == ',').count();

        Some(FunctionCallContext {
            function_name,
            active_parameter: comma_count,
        })
    }

    /// Get signature information for a function
    fn get_function_signatures(&self, function_name: &str) -> Option<Vec<SignatureInformation>> {
        let signature = match function_name {
            "print" => {
                let parameters = vec![
                    ParameterInformation {
                        label: lsp_types::ParameterLabel::Simple("value".to_string()),
                        documentation: Some(lsp_types::Documentation::String("The value to print".to_string())),
                    },
                ];

                SignatureInformation {
                    label: "print(value: any) -> void".to_string(),
                    documentation: Some(lsp_types::Documentation::String("Print a value to the console".to_string())),
                    parameters: Some(parameters),
                    active_parameter: None,
                }
            }
            "len" => {
                let parameters = vec![
                    ParameterInformation {
                        label: lsp_types::ParameterLabel::Simple("collection".to_string()),
                        documentation: Some(lsp_types::Documentation::String("The collection to get length of".to_string())),
                    },
                ];

                SignatureInformation {
                    label: "len(collection) -> Number".to_string(),
                    documentation: Some(lsp_types::Documentation::String("Get the length of a collection".to_string())),
                    parameters: Some(parameters),
                    active_parameter: None,
                }
            }
            "add" => {
                let parameters = vec![
                    ParameterInformation {
                        label: lsp_types::ParameterLabel::Simple("a: Number".to_string()),
                        documentation: Some(lsp_types::Documentation::String("First number".to_string())),
                    },
                    ParameterInformation {
                        label: lsp_types::ParameterLabel::Simple("b: Number".to_string()),
                        documentation: Some(lsp_types::Documentation::String("Second number".to_string())),
                    },
                ];

                SignatureInformation {
                    label: "add(a: Number, b: Number) -> Number".to_string(),
                    documentation: Some(lsp_types::Documentation::String("Add two numbers".to_string())),
                    parameters: Some(parameters),
                    active_parameter: None,
                }
            }
            _ => return None, // Unknown function
        };

        Some(vec![signature])
    }

    /// Calculate which parameter is active based on cursor position
    fn calculate_active_parameter(&self, function_call: &FunctionCallContext, char_pos: usize) -> u32 {
        function_call.active_parameter as u32
    }

    /// Analyze the symbol at the given position for references
    fn analyze_references_symbol(&self, document: &str, position: Position) -> Result<String> {
        // Convert position to byte offset
        let lines: Vec<&str> = document.lines().collect();
        if position.line as usize >= lines.len() {
            return Err(anyhow::anyhow!("Position out of bounds"));
        }

        let line = lines[position.line as usize];
        let char_pos = position.character as usize;

        if char_pos > line.len() {
            return Err(anyhow::anyhow!("Character position out of bounds"));
        }

        // Extract word at position
        let word = self.extract_word_at_position(line, char_pos)
            .ok_or_else(|| anyhow::anyhow!("No word found at position"))?;

        Ok(word)
    }

    /// Find all references to a symbol in the document
    fn find_symbol_references(&self, document: &str, symbol: &str, uri: Url) -> Vec<Location> {
        let mut references = Vec::new();
        let lines: Vec<&str> = document.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            // Find all occurrences of the symbol in this line
            let mut start = 0;
            while let Some(pos) = line[start..].find(symbol) {
                let char_start = start + pos;
                let char_end = char_start + symbol.len();

                // Check if this is a whole word (not part of another word)
                let is_word_boundary = self.is_word_boundary(line, char_start, char_end);

                if is_word_boundary {
                    let start_pos = Position {
                        line: line_idx as u32,
                        character: char_start as u32,
                    };
                    let end_pos = Position {
                        line: line_idx as u32,
                        character: char_end as u32,
                    };

                    let range = Range { start: start_pos, end: end_pos };
                    let location = Location { uri: uri.clone(), range };

                    references.push(location);
                }

                start = char_end;
            }
        }

        references
    }

    /// Check if the symbol at the given range is a whole word
    fn is_word_boundary(&self, line: &str, start: usize, end: usize) -> bool {
        let chars: Vec<char> = line.chars().collect();

        // Check character before start
        let before_ok = if start == 0 {
            true
        } else {
            !chars.get(start - 1).unwrap_or(&' ').is_alphanumeric() && *chars.get(start - 1).unwrap_or(&' ') != '_'
        };

        // Check character after end
        let after_ok = if end >= chars.len() {
            true
        } else {
            !chars.get(end).unwrap_or(&' ').is_alphanumeric() && *chars.get(end).unwrap_or(&' ') != '_'
        };

        before_ok && after_ok
    }

    /// Generate keyword completions
    fn keyword_completions(&self) -> Vec<CompletionItem> {
        vec![
            // Control flow keywords
            CompletionItem {
                label: "if".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Conditional statement".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Conditional execution based on a boolean expression.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "else".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Alternative conditional branch".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Alternative branch for conditional statements.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "match".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Pattern matching".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Pattern matching with exhaustive checking.".to_string(),
                })),
                ..Default::default()
            },

            // Function and type keywords
            CompletionItem {
                label: "fn".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Function declaration".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Declare a function with parameters and return type.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "class".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Class declaration".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Declare a class with methods and properties.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "struct".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Struct declaration".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Declare a data structure with named fields.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "enum".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Enumeration declaration".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Declare an enumeration with variants.".to_string(),
                })),
                ..Default::default()
            },

            // Variable and state keywords
            CompletionItem {
                label: "state".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Reactive state variable".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Declare a reactive state variable that triggers updates.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "let".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Variable binding (deprecated)".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "**Deprecated**: Use direct assignment instead. Variables are immutable by default.".to_string(),
                })),
                deprecated: Some(true),
                ..Default::default()
            },

            // Module and import keywords
            CompletionItem {
                label: "import".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Import statement".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Import modules, types, or functions from other files.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "public".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Public visibility modifier".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Make a declaration visible outside its module.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "async".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Asynchronous function".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Declare an asynchronous function.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "await".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Await asynchronous operation".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Wait for an asynchronous operation to complete.".to_string(),
                })),
                ..Default::default()
            },

            // Error handling
            CompletionItem {
                label: "try".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Try-catch block".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Execute code that might throw exceptions.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "catch".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Exception handler".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Handle exceptions thrown in try blocks.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "throw".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Throw exception".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Throw an exception with an error message.".to_string(),
                })),
                ..Default::default()
            },

            // Reactive system
            CompletionItem {
                label: "computed".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Computed reactive value".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Declare a computed value that updates reactively.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "effect".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Reactive side effect".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Declare a side effect that runs when dependencies change.".to_string(),
                })),
                ..Default::default()
            },
        ]
    }

    /// Generate type completions
    fn type_completions(&self) -> Vec<CompletionItem> {
        vec![
            // Built-in types
            CompletionItem {
                label: "Number".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("64-bit integer type".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "64-bit signed integer type for whole numbers.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "Float".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("64-bit floating point type".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "64-bit floating point type for decimal numbers.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "String".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("UTF-8 string type".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "UTF-8 encoded string type.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "Bool".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("Boolean type".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Boolean type with values `true` and `false`.".to_string(),
                })),
                ..Default::default()
            },

            // Special types
            CompletionItem {
                label: "Option".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("Optional value type".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Represents optional values: `Some(value)` or `None`.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "Result".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("Result type for error handling".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Represents operation results: `Ok(value)` or `Err(error)`.".to_string(),
                })),
                ..Default::default()
            },

            // Collection types
            CompletionItem {
                label: "List".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("Dynamic array type".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Dynamic array that can grow and shrink.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "Set".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("Unique value collection".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Collection of unique values with fast lookup.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "Dict".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("Key-value dictionary".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Dictionary mapping keys to values.".to_string(),
                })),
                ..Default::default()
            },
        ]
    }

    /// Generate function completions
    fn function_completions(&self) -> Vec<CompletionItem> {
        vec![
            // Built-in functions
            CompletionItem {
                label: "print".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("fn print(value: any) -> void".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Print a value to the console.\n\n**Parameters:**\n- `value`: The value to print\n\n**Returns:** Nothing".to_string(),
                })),
                insert_text: Some("print(${1:value})".to_string()),
                insert_text_format: Some(lsp_types::InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "println".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("fn println(value: any) -> void".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Print a value to the console with a newline.\n\n**Parameters:**\n- `value`: The value to print\n\n**Returns:** Nothing".to_string(),
                })),
                insert_text: Some("println(${1:value})".to_string()),
                insert_text_format: Some(lsp_types::InsertTextFormat::SNIPPET),
                ..Default::default()
            },

            // Collection methods
            CompletionItem {
                label: "map".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("fn map<T, U>(self, f: fn(T) -> U) -> List<U>".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Transform each element of a collection.\n\n**Parameters:**\n- `f`: Function to apply to each element\n\n**Returns:** New collection with transformed elements".to_string(),
                })),
                insert_text: Some("map(${1:item} => ${2:transformed})".to_string()),
                insert_text_format: Some(lsp_types::InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "filter".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("fn filter<T>(self, predicate: fn(T) -> Bool) -> List<T>".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Filter elements of a collection.\n\n**Parameters:**\n- `predicate`: Function that returns true for elements to keep\n\n**Returns:** New collection with filtered elements".to_string(),
                })),
                insert_text: Some("filter(${1:item} => ${2:condition})".to_string()),
                insert_text_format: Some(lsp_types::InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "forEach".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("fn forEach<T>(self, action: fn(T) -> void) -> void".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Execute an action for each element.\n\n**Parameters:**\n- `action`: Function to execute for each element\n\n**Returns:** Nothing".to_string(),
                })),
                insert_text: Some("forEach(${1:item} => ${2:action})".to_string()),
                insert_text_format: Some(lsp_types::InsertTextFormat::SNIPPET),
                ..Default::default()
            },

            // String methods
            CompletionItem {
                label: "toUpperCase".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("fn toUpperCase(self) -> String".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Convert string to uppercase.\n\n**Returns:** New uppercase string".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "toLowerCase".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("fn toLowerCase(self) -> String".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Convert string to lowercase.\n\n**Returns:** New lowercase string".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "trim".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("fn trim(self) -> String".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Remove whitespace from both ends.\n\n**Returns:** Trimmed string".to_string(),
                })),
                ..Default::default()
            },
        ]
    }

    /// Generate variable completions
    fn variable_completions(&self) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "value".to_string(),
                kind: Some(CompletionItemKind::VARIABLE),
                detail: Some("Generic value variable".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "A generic variable for storing values.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "result".to_string(),
                kind: Some(CompletionItemKind::VARIABLE),
                detail: Some("Result of an operation".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Variable to store the result of an operation.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "data".to_string(),
                kind: Some(CompletionItemKind::VARIABLE),
                detail: Some("Data variable".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Variable for storing data structures.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "item".to_string(),
                kind: Some(CompletionItemKind::VARIABLE),
                detail: Some("Collection item variable".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Variable representing an item in a collection.".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "index".to_string(),
                kind: Some(CompletionItemKind::VARIABLE),
                detail: Some("Index variable".to_string()),
                documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Variable for loop indices or array positions.".to_string(),
                })),
                ..Default::default()
            },
        ]
    }

    /// Generate basic completions when context is unknown
    fn basic_completions(&self) -> Vec<CompletionItem> {
        let mut completions = Vec::new();

        // Add keywords
        completions.extend(self.keyword_completions());

        // Add common types
        completions.extend(self.type_completions());

        // Add common functions
        completions.extend(self.function_completions());

        completions
    }
}