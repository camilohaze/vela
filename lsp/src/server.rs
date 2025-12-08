//! Language server implementation

use anyhow::Result;
use lsp_server::{Connection, Message, Request, Response};
use lsp_types::{
    InitializeParams, InitializeResult, ServerCapabilities, ServerInfo,
    TextDocumentSyncCapability, TextDocumentSyncKind,
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