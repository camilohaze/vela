//! Language server implementation

use anyhow::Result;
use lsp_server::{Connection, Message, Request, Response};
use lsp_types::{
    InitializeParams, InitializeResult, ServerCapabilities, ServerInfo,
    TextDocumentSyncCapability, TextDocumentSyncKind, CompletionParams,
    CompletionList, CompletionItem, CompletionItemKind, Position,
    CompletionOptions,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

/// Document store for managing open text documents
#[derive(Debug, Default)]
pub struct DocumentStore {
    documents: HashMap<lsp_types::Url, String>,
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

    /// Generate keyword completions
    fn keyword_completions(&self) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "fn".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Function declaration".to_string()),
                documentation: Some(lsp_types::Documentation::String("Declare a function".to_string())),
                ..Default::default()
            },
            CompletionItem {
                label: "let".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Variable declaration".to_string()),
                documentation: Some(lsp_types::Documentation::String("Declare an immutable variable".to_string())),
                ..Default::default()
            },
            CompletionItem {
                label: "state".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Reactive state variable".to_string()),
                documentation: Some(lsp_types::Documentation::String("Declare a mutable reactive variable".to_string())),
                ..Default::default()
            },
            CompletionItem {
                label: "if".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Conditional statement".to_string()),
                documentation: Some(lsp_types::Documentation::String("Conditional execution".to_string())),
                ..Default::default()
            },
            CompletionItem {
                label: "match".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Pattern matching".to_string()),
                documentation: Some(lsp_types::Documentation::String("Pattern matching expression".to_string())),
                ..Default::default()
            },
            CompletionItem {
                label: "class".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Class declaration".to_string()),
                documentation: Some(lsp_types::Documentation::String("Declare a class".to_string())),
                ..Default::default()
            },
            CompletionItem {
                label: "interface".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Interface declaration".to_string()),
                documentation: Some(lsp_types::Documentation::String("Declare an interface".to_string())),
                ..Default::default()
            },
            CompletionItem {
                label: "public".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Public modifier".to_string()),
                documentation: Some(lsp_types::Documentation::String("Make declaration public".to_string())),
                ..Default::default()
            },
            CompletionItem {
                label: "return".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Return statement".to_string()),
                documentation: Some(lsp_types::Documentation::String("Return from function".to_string())),
                ..Default::default()
            },
        ]
    }

    /// Generate type completions
    fn type_completions(&self) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "String".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("String type".to_string()),
                documentation: Some(lsp_types::Documentation::String("Text string type".to_string())),
                ..Default::default()
            },
            CompletionItem {
                label: "Number".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("Number type".to_string()),
                documentation: Some(lsp_types::Documentation::String("Numeric type".to_string())),
                ..Default::default()
            },
            CompletionItem {
                label: "Float".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("Float type".to_string()),
                documentation: Some(lsp_types::Documentation::String("Floating point number type".to_string())),
                ..Default::default()
            },
            CompletionItem {
                label: "Bool".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("Boolean type".to_string()),
                documentation: Some(lsp_types::Documentation::String("True/false type".to_string())),
                ..Default::default()
            },
            CompletionItem {
                label: "void".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("Void type".to_string()),
                documentation: Some(lsp_types::Documentation::String("No return type".to_string())),
                ..Default::default()
            },
        ]
    }

    /// Generate function completions
    fn function_completions(&self) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "print".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("print(value: any) -> void".to_string()),
                documentation: Some(lsp_types::Documentation::String("Print value to console".to_string())),
                ..Default::default()
            },
            CompletionItem {
                label: "len".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("len(collection) -> Number".to_string()),
                documentation: Some(lsp_types::Documentation::String("Get length of collection".to_string())),
                ..Default::default()
            },
        ]
    }

    /// Generate variable completions
    fn variable_completions(&self) -> Vec<CompletionItem> {
        // For now, return empty - this would need semantic analysis
        vec![]
    }

    /// Generate basic completions when context is unknown
    fn basic_completions(&self) -> Vec<CompletionItem> {
        let mut completions = self.keyword_completions();
        completions.extend(self.type_completions());
        completions.extend(self.function_completions());
        completions
    }
}