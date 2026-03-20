pub(crate) mod analysis;
pub(crate) mod completion_helpers;
mod semantic;
mod transport;
mod types;

pub use transport::run_lsp_stdio;
#[cfg(test)]
pub(crate) use transport::run_lsp_stdio_with_io;
pub use types::{
    EditorCompletionItem, JsonRpcError, JsonRpcId, JsonRpcNotification, JsonRpcRequest,
    JsonRpcResponse, LspCompletionContext, LspCompletionItem, LspCompletionList,
    LspCompletionOptions, LspCompletionParams, LspDefinitionParams, LspDidChangeTextDocumentParams,
    LspDidCloseTextDocumentParams, LspDidOpenTextDocumentParams, LspDocumentSymbol,
    LspDocumentSymbolParams, LspHover, LspHoverParams, LspInitializeParams, LspInitializeResult,
    LspPublishDiagnosticsParams, LspServerCapabilities, LspServerInfo,
    LspTextDocumentContentChangeEvent, LspTextDocumentIdentifier, LspTextDocumentItem,
    LspTextDocumentSyncOptions, LspVersionedTextDocumentIdentifier,
};

use crate::{
    dedup_lsp_diagnostics, map_document_workspace, EditorConfig, EditorDocument, EditorDocumentUri, EditorError,
    EditorErrorKind, EditorResult, EditorSession, EditorWorkspaceMapping, LspLocation, LspPosition,
};
use analysis::analyze_document_semantics;
use completion_helpers::completion_context_with_lsp;
use std::sync::Arc;
use transport::from_params;

pub struct EditorLspServer {
    pub session: EditorSession,
}

impl EditorLspServer {
    pub fn new(config: EditorConfig) -> Self {
        Self {
            session: EditorSession::new(config),
        }
    }

    pub fn handle_request(
        &mut self,
        request: JsonRpcRequest,
    ) -> EditorResult<Option<JsonRpcResponse>> {
        match request.method.as_str() {
            "initialize" => Ok(Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(
                    serde_json::to_value(LspInitializeResult {
                        capabilities: LspServerCapabilities {
                            text_document_sync: LspTextDocumentSyncOptions {
                                open_close: true,
                                change: 1,
                            },
                            hover_provider: true,
                            definition_provider: true,
                            document_symbol_provider: true,
                            completion_provider: Some(LspCompletionOptions {
                                trigger_characters: vec![".".to_string()],
                            }),
                        },
                        server_info: LspServerInfo {
                            name: "fol-editor".to_string(),
                            version: env!("CARGO_PKG_VERSION").to_string(),
                        },
                    })
                    .expect("initialize result should serialize"),
                ),
                error: None,
            })),
            "shutdown" => {
                self.session.shutdown_requested = true;
                Ok(Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::Value::Null),
                    error: None,
                }))
            }
            "textDocument/hover" => {
                let params: LspHoverParams = from_params(request.params)?;
                let result = self.hover(
                    &EditorDocumentUri::parse(&params.text_document.uri)?,
                    params.position,
                )?;
                Ok(Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::to_value(result).expect("hover should serialize")),
                    error: None,
                }))
            }
            "textDocument/definition" => {
                let params: LspDefinitionParams = from_params(request.params)?;
                let result = self.definition(
                    &EditorDocumentUri::parse(&params.text_document.uri)?,
                    params.position,
                )?;
                Ok(Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(
                        serde_json::to_value(result).expect("definition result should serialize"),
                    ),
                    error: None,
                }))
            }
            "textDocument/documentSymbol" => {
                let params: LspDocumentSymbolParams = from_params(request.params)?;
                let result =
                    self.document_symbols(&EditorDocumentUri::parse(&params.text_document.uri)?)?;
                Ok(Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(
                        serde_json::to_value(result).expect("document symbols should serialize"),
                    ),
                    error: None,
                }))
            }
            "textDocument/completion" => {
                let params: LspCompletionParams = from_params(request.params)?;
                let result = self.completion(
                    &EditorDocumentUri::parse(&params.text_document.uri)?,
                    params.position,
                    params.context.as_ref(),
                )?;
                Ok(Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(
                        serde_json::to_value(result).expect("completion result should serialize"),
                    ),
                    error: None,
                }))
            }
            _ => Err(EditorError::new(
                EditorErrorKind::InvalidInput,
                format!("unsupported LSP request '{}'", request.method),
            )),
        }
    }

    pub fn handle_notification(
        &mut self,
        notification: JsonRpcNotification,
    ) -> EditorResult<Vec<LspPublishDiagnosticsParams>> {
        use crate::LspPublishDiagnosticsParams;
        match notification.method.as_str() {
            "initialized" => Ok(Vec::new()),
            "exit" => {
                self.session.shutdown_requested = true;
                Ok(Vec::new())
            }
            "textDocument/didOpen" => {
                let params: LspDidOpenTextDocumentParams = from_params(notification.params)?;
                let uri = EditorDocumentUri::parse(&params.text_document.uri)?;
                let document = EditorDocument::new(
                    uri.clone(),
                    params.text_document.version,
                    params.text_document.text,
                )?;
                let mapping =
                    map_document_workspace(document.path.as_path(), &self.session.config)?;
                self.session
                    .mappings
                    .insert(uri.as_str().to_string(), mapping);
                self.session.documents.open(document);
                self.session.semantic_snapshots.remove(uri.as_str());
                let diagnostics = self.publish_diagnostics(&uri)?;
                Ok(vec![diagnostics])
            }
            "textDocument/didChange" => {
                let params: LspDidChangeTextDocumentParams = from_params(notification.params)?;
                let uri = EditorDocumentUri::parse(&params.text_document.uri)?;
                if params.content_changes.is_empty() {
                    return Err(EditorError::new(
                        EditorErrorKind::InvalidInput,
                        "didChange requires at least one content change",
                    ));
                }
                for change in params.content_changes {
                    if let Some(range) = change.range {
                        self.session.documents.apply_incremental_change(
                            &uri,
                            params.text_document.version,
                            range,
                            change.text,
                        )?;
                    } else {
                        self.session.documents.apply_full_change(
                            &uri,
                            params.text_document.version,
                            change.text,
                        )?;
                    }
                }
                self.session.semantic_snapshots.remove(uri.as_str());
                let diagnostics = self.publish_diagnostics(&uri)?;
                Ok(vec![diagnostics])
            }
            "textDocument/didClose" => {
                let params: LspDidCloseTextDocumentParams = from_params(notification.params)?;
                let uri = EditorDocumentUri::parse(&params.text_document.uri)?;
                self.session.documents.close(&uri);
                self.session.mappings.remove(uri.as_str());
                self.session.semantic_snapshots.remove(uri.as_str());
                Ok(vec![LspPublishDiagnosticsParams {
                    uri: uri.as_str().to_string(),
                    diagnostics: Vec::new(),
                }])
            }
            _ => Err(EditorError::new(
                EditorErrorKind::InvalidInput,
                format!("unsupported LSP notification '{}'", notification.method),
            )),
        }
    }

    pub fn publish_diagnostics(
        &mut self,
        uri: &EditorDocumentUri,
    ) -> EditorResult<LspPublishDiagnosticsParams> {
        let document = self.open_document(uri)?.clone();
        let snapshot = self.semantic_snapshot(uri, &document)?;
        let diagnostics = dedup_lsp_diagnostics(snapshot.diagnostics.clone());
        Ok(LspPublishDiagnosticsParams {
            uri: uri.as_str().to_string(),
            diagnostics,
        })
    }

    pub fn hover(
        &mut self,
        uri: &EditorDocumentUri,
        position: LspPosition,
    ) -> EditorResult<Option<LspHover>> {
        let document = self.open_document(uri)?.clone();
        let snapshot = self.semantic_snapshot(uri, &document)?;
        let hit = snapshot
            .reference_at(position)
            .and_then(|reference| snapshot.hover_for_reference(reference));
        Ok(hit)
    }

    pub fn definition(
        &mut self,
        uri: &EditorDocumentUri,
        position: LspPosition,
    ) -> EditorResult<Option<LspLocation>> {
        let document = self.open_document(uri)?.clone();
        let snapshot = self.semantic_snapshot(uri, &document)?;
        Ok(snapshot
            .reference_at(position)
            .and_then(|reference| snapshot.definition_for_reference(reference)))
    }

    pub fn document_symbols(
        &mut self,
        uri: &EditorDocumentUri,
    ) -> EditorResult<Vec<LspDocumentSymbol>> {
        let document = self.open_document(uri)?.clone();
        let snapshot = self.semantic_snapshot(uri, &document)?;
        Ok(snapshot.document_symbols_for_current_path())
    }

    pub fn completion(
        &mut self,
        uri: &EditorDocumentUri,
        position: LspPosition,
        context: Option<&LspCompletionContext>,
    ) -> EditorResult<LspCompletionList> {
        let document = self.open_document(uri)?.clone();
        let completion_context = completion_context_with_lsp(&document, position, context);
        let snapshot = self.semantic_snapshot(uri, &document)?;
        Ok(LspCompletionList {
            is_incomplete: false,
            items: snapshot
                .completion_items(&document, position, completion_context)
                .into_iter()
                .map(|item| LspCompletionItem {
                    label: item.label,
                    kind: item.kind,
                    detail: item.detail,
                    insert_text: item.insert_text,
                })
                .collect(),
        })
    }

    fn open_document(&self, uri: &EditorDocumentUri) -> EditorResult<&EditorDocument> {
        self.session.documents.get(uri).ok_or_else(|| {
            EditorError::new(
                EditorErrorKind::DocumentNotOpen,
                format!("document '{}' is not open", uri.as_str()),
            )
        })
    }

    fn document_mapping(
        &self,
        document: &EditorDocument,
        uri: &EditorDocumentUri,
    ) -> EditorResult<EditorWorkspaceMapping> {
        Ok(self
            .session
            .mappings
            .get(uri.as_str())
            .cloned()
            .unwrap_or(map_document_workspace(
                document.path.as_path(),
                &self.session.config,
            )?))
    }

    fn semantic_snapshot(
        &mut self,
        uri: &EditorDocumentUri,
        document: &EditorDocument,
    ) -> EditorResult<Arc<semantic::SemanticSnapshot>> {
        if let Some(cached) = self.session.semantic_snapshots.get(uri.as_str()) {
            if cached.document_version == document.version {
                return Ok(Arc::clone(&cached.snapshot));
            }
        }

        let mapping = self.document_mapping(document, uri)?;
        let snapshot = Arc::new(analyze_document_semantics(document, &mapping)?);
        self.session.semantic_snapshots.insert(
            uri.as_str().to_string(),
            analysis::CachedSemanticSnapshot {
                document_version: document.version,
                snapshot: Arc::clone(&snapshot),
            },
        );
        Ok(snapshot)
    }
}


#[cfg(test)]
mod tests;
