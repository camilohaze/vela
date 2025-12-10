//! LSP request handlers

use lsp_types::*;
use crate::completion::CompletionProvider;

/// LSP request handlers
pub struct RequestHandlers {
    completion_provider: CompletionProvider,
}

impl RequestHandlers {
    /// Create new request handlers
    pub fn new() -> Self {
        Self {
            completion_provider: CompletionProvider::new(),
        }
    }

    /// Handle completion request
    pub fn handle_completion(&self, params: CompletionParams) -> CompletionList {
        self.completion_provider.get_completions(&params)
    }

    /// Handle hover request
    pub fn handle_hover(&self, _params: HoverParams) -> Option<Hover> {
        // TODO: Implement hover functionality
        None
    }

    /// Handle definition request
    pub fn handle_definition(&self, _params: GotoDefinitionParams) -> Option<GotoDefinitionResponse> {
        // TODO: Implement definition functionality
        None
    }

    /// Handle signature help request
    pub fn handle_signature_help(&self, _params: SignatureHelpParams) -> Option<SignatureHelp> {
        // TODO: Implement signature help functionality
        None
    }

    /// Handle references request
    pub fn handle_references(&self, _params: ReferenceParams) -> Vec<Location> {
        // TODO: Implement references functionality
        Vec::new()
    }
}

impl Default for RequestHandlers {
    fn default() -> Self {
        Self::new()
    }
}