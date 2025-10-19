//! LSP backend implementation.

use kivc::{CompileResult, CompilerConfig, StopAfter, compile_source};
use kivc_diagnostics::KivError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

/// Server state
pub struct ServerState {
    /// Map of document URIs to their content
    documents: HashMap<Url, String>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}

/// LSP backend
pub struct Backend {
    client: Client,
    state: Arc<RwLock<ServerState>>,
}

impl Backend {
    /// Creates a new backend
    pub fn new(client: Client) -> Self {
        Self {
            client,
            state: Arc::new(RwLock::new(ServerState::new())),
        }
    }

    /// Validates a document and publishes diagnostics
    async fn validate_document(&self, uri: &Url, text: &str) {
        let config = CompilerConfig::new().with_stop_after(StopAfter::TypeCheck);

        let (_session, result) = compile_source(uri.to_string(), text.to_string(), config);

        let mut diagnostics = Vec::new();

        match result {
            CompileResult::Failed(diag_collector) => {
                for error in diag_collector.errors() {
                    if let Some(diagnostic) = kiv_error_to_diagnostic(error) {
                        diagnostics.push(diagnostic);
                    }
                }
            }
            CompileResult::StoppedAtHir(_) => {
                // Success, no errors
            }
            _ => {}
        }

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "kiv-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Kiv LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        // Store document
        {
            let mut state = self.state.write().await;
            state.documents.insert(uri.clone(), text.clone());
        }

        // Validate document
        self.validate_document(&uri, &text).await;

        self.client
            .log_message(MessageType::INFO, format!("Opened document: {}", uri))
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        if let Some(change) = params.content_changes.into_iter().next() {
            let text = change.text;

            // Update document
            {
                let mut state = self.state.write().await;
                state.documents.insert(uri.clone(), text.clone());
            }

            // Validate document
            self.validate_document(&uri, &text).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;

        if let Some(text) = {
            let state = self.state.read().await;
            state.documents.get(&uri).cloned()
        } {
            self.validate_document(&uri, &text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        // Remove document
        {
            let mut state = self.state.write().await;
            state.documents.remove(&uri);
        }

        self.client
            .log_message(MessageType::INFO, format!("Closed document: {}", uri))
            .await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let _uri = params.text_document_position_params.text_document.uri;
        let _position = params.text_document_position_params.position;

        // TODO: Implement hover information
        // This would require symbol resolution and type information

        Ok(None)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let _uri = params.text_document_position_params.text_document.uri;
        let _position = params.text_document_position_params.position;

        // TODO: Implement goto definition
        // This would require symbol resolution and tracking definition locations

        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let _uri = params.text_document_position.text_document.uri;
        let _position = params.text_document_position.position;

        // TODO: Implement completion
        // This would require symbol resolution and context analysis

        // For now, return some basic keywords
        let keywords = vec![
            "fun", "let", "mut", "const", "if", "else", "return", "true", "false",
        ];

        let items: Vec<CompletionItem> = keywords
            .into_iter()
            .map(|keyword| CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                ..Default::default()
            })
            .collect();

        Ok(Some(CompletionResponse::Array(items)))
    }
}

/// Converts a KivError to an LSP Diagnostic
fn kiv_error_to_diagnostic(error: &KivError) -> Option<Diagnostic> {
    // Extract span information from the error
    // This is a simplified version - actual implementation would need to
    // properly extract position information from the error

    let message = format!("{}", error);

    // Default range (would need to extract from error span)
    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 0,
        },
    };

    Some(Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        code: None,
        source: Some("kiv".to_string()),
        message,
        related_information: None,
        tags: None,
        code_description: None,
        data: None,
    })
}
