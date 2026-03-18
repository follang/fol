mod analysis;
pub(crate) mod completion_helpers;
mod semantic;
mod transport;
mod types;

pub use transport::run_lsp_stdio;
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
    map_document_workspace, EditorConfig, EditorDocument, EditorDocumentUri, EditorError,
    EditorErrorKind, EditorResult, EditorSession, EditorWorkspaceMapping, LspLocation, LspPosition,
};
use analysis::{analyze_document, analyze_document_semantics};
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
                let diagnostics = self.publish_diagnostics(&uri)?;
                Ok(vec![diagnostics])
            }
            "textDocument/didChange" => {
                let params: LspDidChangeTextDocumentParams = from_params(notification.params)?;
                let uri = EditorDocumentUri::parse(&params.text_document.uri)?;
                let text = params
                    .content_changes
                    .last()
                    .map(|change| change.text.clone())
                    .ok_or_else(|| {
                        EditorError::new(
                            EditorErrorKind::InvalidInput,
                            "didChange requires at least one content change",
                        )
                    })?;
                self.session.documents.apply_full_change(
                    &uri,
                    params.text_document.version,
                    text,
                )?;
                let diagnostics = self.publish_diagnostics(&uri)?;
                Ok(vec![diagnostics])
            }
            "textDocument/didClose" => {
                let params: LspDidCloseTextDocumentParams = from_params(notification.params)?;
                let uri = EditorDocumentUri::parse(&params.text_document.uri)?;
                self.session.documents.close(&uri);
                self.session.mappings.remove(uri.as_str());
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
        &self,
        uri: &EditorDocumentUri,
    ) -> EditorResult<LspPublishDiagnosticsParams> {
        let document = self.open_document(uri)?;
        let mapping = self.document_mapping(document, uri)?;
        let diagnostics = analyze_document(document, &mapping)?;
        Ok(LspPublishDiagnosticsParams {
            uri: uri.as_str().to_string(),
            diagnostics,
        })
    }

    pub fn hover(
        &self,
        uri: &EditorDocumentUri,
        position: LspPosition,
    ) -> EditorResult<Option<LspHover>> {
        let document = self.open_document(uri)?;
        let mapping = self.document_mapping(document, uri)?;
        let snapshot = analyze_document_semantics(document, &mapping)?;
        let hit = snapshot
            .reference_at(position)
            .and_then(|reference| snapshot.hover_for_reference(reference));
        Ok(hit)
    }

    pub fn definition(
        &self,
        uri: &EditorDocumentUri,
        position: LspPosition,
    ) -> EditorResult<Option<LspLocation>> {
        let document = self.open_document(uri)?;
        let mapping = self.document_mapping(document, uri)?;
        let snapshot = analyze_document_semantics(document, &mapping)?;
        Ok(snapshot
            .reference_at(position)
            .and_then(|reference| snapshot.definition_for_reference(reference)))
    }

    pub fn document_symbols(
        &self,
        uri: &EditorDocumentUri,
    ) -> EditorResult<Vec<LspDocumentSymbol>> {
        let document = self.open_document(uri)?;
        let mapping = self.document_mapping(document, uri)?;
        let snapshot = analyze_document_semantics(document, &mapping)?;
        Ok(snapshot.document_symbols_for_current_path())
    }

    pub fn completion(
        &self,
        uri: &EditorDocumentUri,
        position: LspPosition,
    ) -> EditorResult<LspCompletionList> {
        let document = self.open_document(uri)?;
        let mapping = self.document_mapping(document, uri)?;
        let snapshot = analyze_document_semantics(document, &mapping)?;
        Ok(LspCompletionList {
            is_incomplete: false,
            items: snapshot
                .plain_completion_items(document, position)
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
}


#[cfg(test)]
mod tests;
