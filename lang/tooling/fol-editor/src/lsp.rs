use crate::{
    diagnostic_to_lsp, map_document_workspace, materialize_analysis_overlay, EditorConfig,
    location_to_range, EditorDocument, EditorDocumentUri, EditorError, EditorErrorKind,
    EditorResult, EditorSession, EditorWorkspaceMapping, LspDiagnostic, LspLocation, LspPosition,
    LspRange,
};
use fol_diagnostics::Diagnostic;
use fol_diagnostics::ToDiagnostic;
use fol_package::{PackageError, PackageSession, PackageSourceKind};
use fol_parser::ast::{AstParser, ParseError};
use fol_resolver::{Resolver, ResolverError};
use fol_stream::{FileStream, Source, SourceType};
use fol_typecheck::{TypecheckError, Typechecker};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum JsonRpcId {
    Number(i64),
    String(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: JsonRpcId,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: JsonRpcId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LspServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LspTextDocumentSyncOptions {
    pub open_close: bool,
    pub change: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LspServerCapabilities {
    pub text_document_sync: LspTextDocumentSyncOptions,
    pub hover_provider: bool,
    pub definition_provider: bool,
    pub document_symbol_provider: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completion_provider: Option<LspCompletionOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LspCompletionOptions {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trigger_characters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LspInitializeParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LspInitializeResult {
    pub capabilities: LspServerCapabilities,
    pub server_info: LspServerInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LspTextDocumentItem {
    pub uri: String,
    pub language_id: String,
    pub version: i32,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LspVersionedTextDocumentIdentifier {
    pub uri: String,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LspTextDocumentContentChangeEvent {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LspDidOpenTextDocumentParams {
    pub text_document: LspTextDocumentItem,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LspDidChangeTextDocumentParams {
    pub text_document: LspVersionedTextDocumentIdentifier,
    pub content_changes: Vec<LspTextDocumentContentChangeEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LspDidCloseTextDocumentParams {
    pub text_document: LspVersionedTextDocumentIdentifier,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LspPublishDiagnosticsParams {
    pub uri: String,
    pub diagnostics: Vec<LspDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LspTextDocumentIdentifier {
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LspHoverParams {
    pub text_document: LspTextDocumentIdentifier,
    pub position: LspPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LspDefinitionParams {
    pub text_document: LspTextDocumentIdentifier,
    pub position: LspPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LspDocumentSymbolParams {
    pub text_document: LspTextDocumentIdentifier,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LspCompletionContext {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_character: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LspCompletionParams {
    pub text_document: LspTextDocumentIdentifier,
    pub position: LspPosition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<LspCompletionContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LspCompletionList {
    pub is_incomplete: bool,
    pub items: Vec<LspCompletionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LspCompletionItem {
    pub label: String,
    pub kind: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insert_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EditorCompletionItem {
    pub label: String,
    pub kind: u8,
    pub detail: Option<String>,
    pub insert_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LspHover {
    pub contents: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<LspRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LspDocumentSymbol {
    pub name: String,
    pub kind: u8,
    pub range: LspRange,
    pub selection_range: LspRange,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<LspDocumentSymbol>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorLspServer {
    pub session: EditorSession,
}

impl EditorLspServer {
    pub fn new(config: EditorConfig) -> Self {
        Self {
            session: EditorSession::new(config),
        }
    }

    pub fn handle_request(&mut self, request: JsonRpcRequest) -> EditorResult<Option<JsonRpcResponse>> {
        match request.method.as_str() {
            "initialize" => Ok(Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(serde_json::to_value(LspInitializeResult {
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
                .expect("initialize result should serialize")),
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
                let result = self.document_symbols(&EditorDocumentUri::parse(&params.text_document.uri)?)?;
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
        match notification.method.as_str() {
            "initialized" => Ok(Vec::new()),
            "exit" => {
                self.session.shutdown_requested = true;
                Ok(Vec::new())
            }
            "textDocument/didOpen" => {
                let params: LspDidOpenTextDocumentParams = from_params(notification.params)?;
                let uri = EditorDocumentUri::parse(&params.text_document.uri)?;
                let document =
                    EditorDocument::new(uri.clone(), params.text_document.version, params.text_document.text)?;
                let mapping = map_document_workspace(document.path.as_path(), &self.session.config)?;
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
                self.session
                    .documents
                    .apply_full_change(&uri, params.text_document.version, text)?;
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

    pub fn hover(&self, uri: &EditorDocumentUri, position: LspPosition) -> EditorResult<Option<LspHover>> {
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
                .local_completion_items(position)
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
        Ok(self.session
            .mappings
            .get(uri.as_str())
            .cloned()
            .unwrap_or(map_document_workspace(document.path.as_path(), &self.session.config)?))
    }
}

pub fn run_lsp_stdio(config: EditorConfig) -> EditorResult<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    run_lsp_stdio_with_io(stdin.lock(), stdout.lock(), config)
}

pub(crate) fn run_lsp_stdio_with_io<R: BufRead, W: Write>(
    mut reader: R,
    mut writer: W,
    config: EditorConfig,
) -> EditorResult<()> {
    let mut server = EditorLspServer::new(config);

    while let Some(payload) = read_jsonrpc_payload(&mut reader)? {
        let value: serde_json::Value = serde_json::from_slice(&payload).map_err(|error| {
            EditorError::new(
                EditorErrorKind::InvalidInput,
                format!("failed to decode JSON-RPC payload: {error}"),
            )
        })?;

        if value.get("id").is_some() {
            let request: JsonRpcRequest = serde_json::from_value(value).map_err(|error| {
                EditorError::new(
                    EditorErrorKind::InvalidInput,
                    format!("failed to decode JSON-RPC request: {error}"),
                )
            })?;
            let id = request.id.clone();
            match server.handle_request(request) {
                Ok(Some(response)) => {
                    write_jsonrpc_message(&mut writer, &response)?;
                }
                Ok(None) => {}
                Err(error) => {
                    let response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32603,
                            message: error.message,
                        }),
                    };
                    write_jsonrpc_message(&mut writer, &response)?;
                }
            }
        } else {
            let notification: JsonRpcNotification =
                serde_json::from_value(value).map_err(|error| {
                    EditorError::new(
                        EditorErrorKind::InvalidInput,
                        format!("failed to decode JSON-RPC notification: {error}"),
                    )
                })?;
            let should_exit = notification.method == "exit";
            let diagnostics = server.handle_notification(notification)?;
            for diagnostics in diagnostics {
                write_jsonrpc_message(
                    &mut writer,
                    &JsonRpcNotification {
                        jsonrpc: "2.0".to_string(),
                        method: "textDocument/publishDiagnostics".to_string(),
                        params: Some(
                            serde_json::to_value(diagnostics)
                                .expect("publish diagnostics should serialize"),
                        ),
                    },
                )?;
            }
            if should_exit || server.session.shutdown_requested {
                break;
            }
        }
    }

    Ok(())
}

fn read_jsonrpc_payload(reader: &mut impl BufRead) -> EditorResult<Option<Vec<u8>>> {
    let mut content_length = None;
    let mut line = String::new();

    loop {
        line.clear();
        let read = reader.read_line(&mut line).map_err(|error| {
            EditorError::new(
                EditorErrorKind::Internal,
                format!("failed to read LSP header line: {error}"),
            )
        })?;
        if read == 0 {
            return Ok(None);
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(value) = trimmed.strip_prefix("Content-Length:") {
            let length = value.trim().parse::<usize>().map_err(|error| {
                EditorError::new(
                    EditorErrorKind::InvalidInput,
                    format!("invalid Content-Length header: {error}"),
                )
            })?;
            content_length = Some(length);
        }
    }

    let content_length = content_length.ok_or_else(|| {
        EditorError::new(
            EditorErrorKind::InvalidInput,
            "missing Content-Length header in LSP message",
        )
    })?;

    let mut payload = vec![0; content_length];
    reader.read_exact(&mut payload).map_err(|error| {
        EditorError::new(
            EditorErrorKind::Internal,
            format!("failed to read LSP payload: {error}"),
        )
    })?;
    Ok(Some(payload))
}

fn write_jsonrpc_message(writer: &mut impl Write, value: &impl Serialize) -> EditorResult<()> {
    let body = serde_json::to_vec(value).map_err(|error| {
        EditorError::new(
            EditorErrorKind::Internal,
            format!("failed to encode JSON-RPC payload: {error}"),
        )
    })?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len()).map_err(|error| {
        EditorError::new(
            EditorErrorKind::Internal,
            format!("failed to write LSP header: {error}"),
        )
    })?;
    writer.write_all(&body).map_err(|error| {
        EditorError::new(
            EditorErrorKind::Internal,
            format!("failed to write LSP payload: {error}"),
        )
    })?;
    writer.flush().map_err(|error| {
        EditorError::new(
            EditorErrorKind::Internal,
            format!("failed to flush LSP payload: {error}"),
        )
    })?;
    Ok(())
}

fn from_params<T: serde::de::DeserializeOwned>(params: Option<serde_json::Value>) -> EditorResult<T> {
    serde_json::from_value(params.unwrap_or(serde_json::Value::Null)).map_err(|error| {
        EditorError::new(
            EditorErrorKind::InvalidInput,
            format!("invalid LSP params: {error}"),
        )
    })
}

fn analyze_document(
    document: &EditorDocument,
    mapping: &EditorWorkspaceMapping,
) -> EditorResult<Vec<LspDiagnostic>> {
    let snapshot = analyze_document_semantics(document, mapping)?;
    Ok(snapshot.diagnostics)
}

struct SemanticSnapshot {
    analyzed_path: Option<PathBuf>,
    diagnostics: Vec<LspDiagnostic>,
    typed_workspace: Option<fol_typecheck::TypedWorkspace>,
}

impl SemanticSnapshot {
    fn local_completion_items(&self, position: LspPosition) -> Vec<EditorCompletionItem> {
        let Some((program, scope_id)) = self.scope_at_position(position) else {
            return Vec::new();
        };
        let mut items = Vec::new();
        let mut cursor = Some(scope_id);
        while let Some(current_scope_id) = cursor {
            for symbol in program.symbols_in_scope(current_scope_id) {
                if !matches!(
                    symbol.kind,
                    fol_resolver::SymbolKind::ValueBinding
                        | fol_resolver::SymbolKind::LabelBinding
                        | fol_resolver::SymbolKind::DestructureBinding
                        | fol_resolver::SymbolKind::LoopBinder
                        | fol_resolver::SymbolKind::RollingBinder
                        | fol_resolver::SymbolKind::Capture
                ) {
                    continue;
                }
                items.push(EditorCompletionItem {
                    label: symbol.name.clone(),
                    kind: symbol_kind_code(symbol.kind),
                    detail: Some(render_symbol_kind(symbol.kind).to_string()),
                    insert_text: None,
                });
            }
            cursor = program.scope(current_scope_id).and_then(|scope| scope.parent);
        }
        items
    }

    fn scope_at_position(
        &self,
        position: LspPosition,
    ) -> Option<(&fol_resolver::ResolvedProgram, fol_resolver::ScopeId)> {
        let typed = self.typed_workspace.as_ref()?;
        let analyzed_path = self.analyzed_path.as_ref()?;
        for package in typed.packages() {
            let program = package.program.resolved();
            let Some(syntax_id) = syntax_at_position(program, analyzed_path.as_path(), position) else {
                continue;
            };
            if let Some(scope_id) = program.scope_for_syntax(syntax_id) {
                return Some((program, scope_id));
            }
        }
        None
    }

    fn reference_at(&self, position: LspPosition) -> Option<&fol_resolver::ResolvedReference> {
        let typed = self.typed_workspace.as_ref()?;
        let analyzed_path = self.analyzed_path.as_ref()?;
        let needle = typed
            .packages()
            .find_map(|package| {
                let program = package.program.resolved();
                let syntax_id = syntax_at_position(program, analyzed_path.as_path(), position)?;
                program
                    .all_references()
                    .find(|reference| reference.syntax_id == Some(syntax_id))
            })?;
        Some(needle)
    }

    fn hover_for_reference(&self, reference: &fol_resolver::ResolvedReference) -> Option<LspHover> {
        let typed = self.typed_workspace.as_ref()?;
        for package in typed.packages() {
            let program = &package.program;
            let resolved = program.resolved();
            if let Some(symbol_id) = reference.resolved {
                let symbol = resolved.symbol(symbol_id)?;
                let typed_symbol = program.typed_symbol(symbol_id);
                let origin = symbol.origin.as_ref()?;
                let type_summary = typed_symbol
                    .and_then(|typed_symbol| typed_symbol.declared_type)
                    .map(|type_id| render_checked_type(program.type_table(), type_id))
                    .unwrap_or_else(|| "unknown".to_string());
                return Some(LspHover {
                    contents: format!("{} {}: {}", render_symbol_kind(symbol.kind), symbol.name, type_summary),
                    range: Some(location_to_range(&fol_diagnostics::DiagnosticLocation {
                        file: origin.file.clone(),
                        line: origin.line,
                        column: origin.column,
                        length: Some(origin.length),
                    })),
                });
            }
        }
        None
    }

    fn definition_for_reference(
        &self,
        reference: &fol_resolver::ResolvedReference,
    ) -> Option<LspLocation> {
        let typed = self.typed_workspace.as_ref()?;
        for package in typed.packages() {
            let resolved = package.program.resolved();
            if let Some(symbol_id) = reference.resolved {
                let symbol = resolved.symbol(symbol_id)?;
                let origin = symbol.origin.as_ref()?;
                let file = origin.file.as_ref()?;
                return Some(LspLocation {
                    uri: format!("file://{file}"),
                    range: location_to_range(&fol_diagnostics::DiagnosticLocation {
                        file: Some(file.clone()),
                        line: origin.line,
                        column: origin.column,
                        length: Some(origin.length),
                    }),
                });
            }
        }
        None
    }

    fn document_symbols_for_current_path(&self) -> Vec<LspDocumentSymbol> {
        let typed = match &self.typed_workspace {
            Some(typed) => typed,
            None => return Vec::new(),
        };
        let Some(analyzed_path) = &self.analyzed_path else {
            return Vec::new();
        };
        let path_text = analyzed_path.to_string_lossy();
        let mut symbols = Vec::new();
        for package in typed.packages() {
            let program = package.program.resolved();
            for symbol in program.all_symbols() {
                let Some(origin) = &symbol.origin else { continue };
                let Some(file) = &origin.file else { continue };
                if file != &path_text {
                    continue;
                }
                let range = location_to_range(&fol_diagnostics::DiagnosticLocation {
                    file: Some(file.clone()),
                    line: origin.line,
                    column: origin.column,
                    length: Some(origin.length),
                });
                symbols.push(LspDocumentSymbol {
                    name: symbol.name.clone(),
                    kind: symbol_kind_code(symbol.kind),
                    range,
                    selection_range: range,
                    children: Vec::new(),
                });
            }
        }
        symbols.sort_by(|left, right| {
            left.range
                .start
                .line
                .cmp(&right.range.start.line)
                .then(left.range.start.character.cmp(&right.range.start.character))
                .then(left.name.cmp(&right.name))
        });
        nest_document_symbols(symbols)
    }
}

fn nest_document_symbols(symbols: Vec<LspDocumentSymbol>) -> Vec<LspDocumentSymbol> {
    fn insert(into: &mut Vec<LspDocumentSymbol>, symbol: LspDocumentSymbol) {
        if let Some(parent) = into
            .iter_mut()
            .rev()
            .find(|candidate| range_contains(&candidate.range, &symbol.range))
        {
            insert(&mut parent.children, symbol);
        } else {
            into.push(symbol);
        }
    }

    let mut nested = Vec::new();
    for symbol in symbols {
        insert(&mut nested, symbol);
    }
    nested
}

fn range_contains(parent: &LspRange, child: &LspRange) -> bool {
    let parent_start = (parent.start.line, parent.start.character);
    let parent_end = (parent.end.line, parent.end.character);
    let child_start = (child.start.line, child.start.character);
    let child_end = (child.end.line, child.end.character);

    parent_start <= child_start
        && child_end <= parent_end
        && (parent_start != child_start || parent_end != child_end)
}

fn analyze_document_semantics(
    document: &EditorDocument,
    mapping: &EditorWorkspaceMapping,
) -> EditorResult<SemanticSnapshot> {
    let overlay = materialize_analysis_overlay(mapping, document)?;
    if let Some(package_root) = overlay.package_root() {
        let parser_diags = parse_directory_diagnostics(package_root)?
            .into_iter()
            .filter(|diagnostic| diagnostic_targets_path(diagnostic, overlay.document_path()))
            .map(|diagnostic| diagnostic_to_lsp(&diagnostic))
            .collect::<Vec<_>>();
        if !parser_diags.is_empty() {
            return Ok(SemanticSnapshot {
                analyzed_path: Some(overlay.document_path().to_path_buf()),
                diagnostics: parser_diags,
                typed_workspace: None,
            });
        }

        let mut package_session = PackageSession::new();
        let prepared = match package_session.load_directory_package(package_root, PackageSourceKind::Entry) {
            Ok(prepared) => prepared,
            Err(error) => {
                return Ok(SemanticSnapshot {
                    analyzed_path: Some(overlay.document_path().to_path_buf()),
                    diagnostics: vec![diagnostic_to_lsp(&error.to_diagnostic())],
                    typed_workspace: None,
                })
            }
        };

        let mut resolver = Resolver::new();
        let resolved = match resolver.resolve_prepared_workspace(prepared) {
            Ok(resolved) => resolved,
            Err(errors) => {
                return Ok(SemanticSnapshot {
                    analyzed_path: Some(overlay.document_path().to_path_buf()),
                    diagnostics: errors
                        .iter()
                        .map(|error| error.to_diagnostic())
                        .filter(|diagnostic| diagnostic_targets_path(diagnostic, overlay.document_path()))
                        .map(|diagnostic| diagnostic_to_lsp(&diagnostic))
                        .collect(),
                    typed_workspace: None,
                })
            }
        };

        let mut typechecker = Typechecker::new();
        match typechecker.check_resolved_workspace(resolved) {
            Ok(typed_workspace) => Ok(SemanticSnapshot {
                analyzed_path: Some(overlay.document_path().to_path_buf()),
                diagnostics: Vec::new(),
                typed_workspace: Some(typed_workspace),
            }),
            Err(errors) => Ok(SemanticSnapshot {
                analyzed_path: Some(overlay.document_path().to_path_buf()),
                diagnostics: errors
                    .iter()
                    .map(|error| error.to_diagnostic())
                    .filter(|diagnostic| diagnostic_targets_path(diagnostic, overlay.document_path()))
                    .map(|diagnostic| diagnostic_to_lsp(&diagnostic))
                    .collect(),
                typed_workspace: None,
            }),
        }
    } else {
        Ok(SemanticSnapshot {
            analyzed_path: Some(mapping.document_path.clone()),
            diagnostics: parse_single_file_diagnostics(&mapping.document_path, &document.text)?
                .into_iter()
                .filter(|diagnostic| diagnostic_targets_path(diagnostic, &mapping.document_path))
                .map(|diagnostic| diagnostic_to_lsp(&diagnostic))
                .collect(),
            typed_workspace: None,
        })
    }
}

fn diagnostic_targets_path(diagnostic: &Diagnostic, path: &Path) -> bool {
    let path_text = path.to_string_lossy();
    diagnostic
        .primary_location()
        .and_then(|location| location.file.as_ref())
        .map(|file| file == &path_text)
        .or_else(|| {
            diagnostic
                .labels
                .first()
                .and_then(|label| label.location.file.as_ref())
                .map(|file| file == &path_text)
        })
        .unwrap_or(false)
}

fn parse_single_file_diagnostics(path: &Path, text: &str) -> EditorResult<Vec<Diagnostic>> {
    let root = std::env::temp_dir().join(format!(
        "fol_editor_parse_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).map_err(|error| {
        EditorError::new(
            EditorErrorKind::Internal,
            format!("failed to create parser temp root '{}': {error}", root.display()),
        )
    })?;
    let file = root.join(
        path.file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("main.fol")),
    );
    std::fs::write(&file, text).map_err(|error| {
        EditorError::new(
            EditorErrorKind::Internal,
            format!("failed to write parser temp file '{}': {error}", file.display()),
        )
    })?;
    let diagnostics = parse_directory_diagnostics(&root)?;
    let _ = std::fs::remove_dir_all(&root);
    Ok(diagnostics)
}

fn parse_directory_diagnostics(root: &Path) -> EditorResult<Vec<Diagnostic>> {
    let root_str = root.to_str().ok_or_else(|| {
        EditorError::new(
            EditorErrorKind::InvalidDocumentPath,
            format!("analysis root '{}' is not valid UTF-8", root.display()),
        )
    })?;
    let display_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("root");
    let sources = Source::init_with_package(root_str, SourceType::Folder, display_name).map_err(|error| {
        EditorError::new(
            EditorErrorKind::Internal,
            format!("failed to initialize analysis sources from '{}': {error}", root.display()),
        )
    })?;
    let mut stream = FileStream::from_sources(sources).map_err(|error| {
        EditorError::new(
            EditorErrorKind::Internal,
            format!("failed to read analysis sources from '{}': {error}", root.display()),
        )
    })?;
    let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
    let mut parser = AstParser::new();

    match parser.parse_package(&mut lexer) {
        Ok(_) => Ok(Vec::new()),
        Err(errors) => Ok(errors
            .into_iter()
            .map(|error| glitch_to_diagnostic(error.as_ref()))
            .collect()),
    }
}

fn glitch_to_diagnostic(error: &dyn fol_types::Glitch) -> Diagnostic {
    if let Some(parse_error) = error.as_any().downcast_ref::<ParseError>() {
        return parse_error.to_diagnostic();
    }
    if let Some(package_error) = error.as_any().downcast_ref::<PackageError>() {
        return package_error.to_diagnostic();
    }
    if let Some(resolver_error) = error.as_any().downcast_ref::<ResolverError>() {
        return resolver_error.to_diagnostic();
    }
    if let Some(typecheck_error) = error.as_any().downcast_ref::<TypecheckError>() {
        return typecheck_error.to_diagnostic();
    }
    fol_diagnostics::Diagnostic::error("E9999", error.to_string())
}

fn syntax_at_position(
    program: &fol_resolver::ResolvedProgram,
    path: &Path,
    position: LspPosition,
) -> Option<fol_parser::ast::SyntaxNodeId> {
    let path_text = path.to_string_lossy();
    let mut best: Option<(fol_parser::ast::SyntaxNodeId, usize)> = None;
    for index in 0..program.syntax_index().len() {
        let syntax_id = fol_parser::ast::SyntaxNodeId(index);
        let Some(origin) = program.syntax_index().origin(syntax_id) else {
            continue;
        };
        let Some(file) = &origin.file else {
            continue;
        };
        if file != &path_text {
            continue;
        }
        let start_line = origin.line.saturating_sub(1) as u32;
        let start_character = origin.column.saturating_sub(1) as u32;
        let end_character = start_character + origin.length.max(1) as u32;
        let contains = position.line == start_line
            && position.character >= start_character
            && position.character <= end_character;
        if contains {
            match best {
                Some((_, current_len)) if current_len <= origin.length => {}
                _ => best = Some((syntax_id, origin.length)),
            }
        }
    }
    best.map(|(syntax_id, _)| syntax_id)
}

fn render_symbol_kind(kind: fol_resolver::SymbolKind) -> &'static str {
    match kind {
        fol_resolver::SymbolKind::Type => "type",
        fol_resolver::SymbolKind::Alias => "alias",
        fol_resolver::SymbolKind::Routine => "routine",
        fol_resolver::SymbolKind::Definition => "definition",
        fol_resolver::SymbolKind::ValueBinding
        | fol_resolver::SymbolKind::LabelBinding
        | fol_resolver::SymbolKind::DestructureBinding => "binding",
        fol_resolver::SymbolKind::Parameter => "parameter",
        fol_resolver::SymbolKind::Capture => "capture",
        fol_resolver::SymbolKind::ImportAlias => "namespace",
        fol_resolver::SymbolKind::Segment => "segment",
        fol_resolver::SymbolKind::Implementation => "implementation",
        fol_resolver::SymbolKind::Standard => "standard",
        fol_resolver::SymbolKind::GenericParameter => "parameter",
        fol_resolver::SymbolKind::LoopBinder => "binding",
        fol_resolver::SymbolKind::RollingBinder => "binding",
    }
}

fn symbol_kind_code(kind: fol_resolver::SymbolKind) -> u8 {
    match kind {
        fol_resolver::SymbolKind::Routine | fol_resolver::SymbolKind::Definition => 12,
        fol_resolver::SymbolKind::Type | fol_resolver::SymbolKind::Alias => 5,
        fol_resolver::SymbolKind::ImportAlias => 3,
        fol_resolver::SymbolKind::ValueBinding
        | fol_resolver::SymbolKind::LabelBinding
        | fol_resolver::SymbolKind::DestructureBinding
        | fol_resolver::SymbolKind::Parameter
        | fol_resolver::SymbolKind::Capture
        | fol_resolver::SymbolKind::GenericParameter
        | fol_resolver::SymbolKind::LoopBinder
        | fol_resolver::SymbolKind::RollingBinder => 13,
        fol_resolver::SymbolKind::Segment => 2,
        fol_resolver::SymbolKind::Implementation | fol_resolver::SymbolKind::Standard => 6,
    }
}

fn render_checked_type(
    table: &fol_typecheck::TypeTable,
    type_id: fol_typecheck::CheckedTypeId,
) -> String {
    match table.get(type_id) {
        Some(fol_typecheck::CheckedType::Builtin(builtin)) => match builtin {
            fol_typecheck::BuiltinType::Int => "int".to_string(),
            fol_typecheck::BuiltinType::Float => "flt".to_string(),
            fol_typecheck::BuiltinType::Bool => "bol".to_string(),
            fol_typecheck::BuiltinType::Char => "chr".to_string(),
            fol_typecheck::BuiltinType::Str => "str".to_string(),
            fol_typecheck::BuiltinType::Never => "never".to_string(),
        },
        Some(fol_typecheck::CheckedType::Declared { name, .. }) => name.clone(),
        Some(fol_typecheck::CheckedType::Optional { inner }) => {
            format!("opt[{}]", render_checked_type(table, *inner))
        }
        Some(fol_typecheck::CheckedType::Error { inner }) => inner
            .map(|inner| format!("err[{}]", render_checked_type(table, inner)))
            .unwrap_or_else(|| "err[]".to_string()),
        Some(fol_typecheck::CheckedType::Array { element_type, .. }) => {
            format!("[{}]", render_checked_type(table, *element_type))
        }
        Some(fol_typecheck::CheckedType::Vector { element_type }) => {
            format!("vec[{}]", render_checked_type(table, *element_type))
        }
        Some(fol_typecheck::CheckedType::Sequence { element_type }) => {
            format!("seq[{}]", render_checked_type(table, *element_type))
        }
        Some(fol_typecheck::CheckedType::Set { member_types }) => format!(
            "set[{}]",
            member_types
                .iter()
                .map(|member| render_checked_type(table, *member))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Some(fol_typecheck::CheckedType::Map { key_type, value_type }) => format!(
            "map[{}, {}]",
            render_checked_type(table, *key_type),
            render_checked_type(table, *value_type)
        ),
        Some(fol_typecheck::CheckedType::Routine(routine)) => {
            let params = routine
                .params
                .iter()
                .map(|param| render_checked_type(table, *param))
                .collect::<Vec<_>>()
                .join(", ");
            let returns = routine
                .return_type
                .map(|return_type| render_checked_type(table, return_type))
                .unwrap_or_else(|| "void".to_string());
            match routine.error_type {
                Some(error_type) => format!("fun({params}): {returns} / {}", render_checked_type(table, error_type)),
                None => format!("fun({params}): {returns}"),
            }
        }
        Some(other) => format!("{other:?}"),
        None => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EditorLspServer, JsonRpcId, JsonRpcNotification, JsonRpcRequest,
        LspCompletionList, LspCompletionParams, LspDefinitionParams,
        LspDidChangeTextDocumentParams, LspDidCloseTextDocumentParams,
        LspDidOpenTextDocumentParams, LspDocumentSymbolParams, LspHover, LspHoverParams,
        LspInitializeResult, LspLocation, LspPosition, LspPublishDiagnosticsParams,
        LspTextDocumentContentChangeEvent, LspTextDocumentIdentifier, LspTextDocumentItem,
        LspVersionedTextDocumentIdentifier,
    };
    use crate::EditorConfig;
    use std::fs;
    use std::path::PathBuf;

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "fol_editor_lsp_{}_{}_{}",
            label,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time should be after epoch")
                .as_nanos()
        ))
    }

    fn sample_package_root(label: &str) -> (PathBuf, String) {
        let root = temp_root(label);
        let src = root.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
        fs::write(root.join("build.fol"), "def root: loc = \"src\"\n").unwrap();
        let file = src.join("main.fol");
        fs::write(&file, "fun[] main(): int = {\n    return 0\n}\n").unwrap();
        let uri = format!("file://{}", file.display());
        (root, uri)
    }

    fn sample_loc_workspace_root(label: &str) -> (PathBuf, String) {
        let root = temp_root(label);
        let app_src = root.join("app/src");
        let shared_src = root.join("shared/src");
        fs::create_dir_all(&app_src).unwrap();
        fs::create_dir_all(&shared_src).unwrap();

        fs::write(root.join("app/package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();
        fs::write(root.join("app/build.fol"), "def root: loc = \"src\"\n").unwrap();
        fs::write(root.join("shared/package.yaml"), "name: shared\nversion: 0.1.0\n").unwrap();
        fs::write(root.join("shared/build.fol"), "def root: loc = \"src\"\n").unwrap();

        fs::write(
            root.join("app/src/main.fol"),
            "use shared: loc = {\"../shared\"};\n\nfun[] main(): int = {\n    return shared.helper()\n}\n",
        )
        .unwrap();
        fs::write(
            root.join("shared/src/lib.fol"),
            "fun[exp] helper(): int = {\n    return 9\n}\n",
        )
        .unwrap();

        let uri = format!("file://{}", root.join("app/src/main.fol").display());
        (root, uri)
    }

    fn open_document(server: &mut EditorLspServer, uri: String, text: &str) -> Vec<LspPublishDiagnosticsParams> {
        server
            .handle_notification(JsonRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "textDocument/didOpen".to_string(),
                params: Some(
                    serde_json::to_value(LspDidOpenTextDocumentParams {
                        text_document: LspTextDocumentItem {
                            uri,
                            language_id: "fol".to_string(),
                            version: 1,
                            text: text.to_string(),
                        },
                    })
                    .unwrap(),
                ),
            })
            .unwrap()
    }

    #[test]
    fn lsp_server_handles_initialize_shutdown_and_exit() {
        let mut server = EditorLspServer::new(EditorConfig::default());

        let initialize = server
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: JsonRpcId::Number(1),
                method: "initialize".to_string(),
                params: Some(serde_json::json!({})),
            })
            .unwrap()
            .unwrap();
        let result: LspInitializeResult = serde_json::from_value(initialize.result.unwrap()).unwrap();
        assert!(result.capabilities.text_document_sync.open_close);
        let completion_provider = result
            .capabilities
            .completion_provider
            .expect("completion provider should be advertised");
        assert_eq!(completion_provider.trigger_characters, vec![".".to_string()]);

        let shutdown = server
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: JsonRpcId::Number(2),
                method: "shutdown".to_string(),
                params: None,
            })
            .unwrap()
            .unwrap();
        assert_eq!(shutdown.id, JsonRpcId::Number(2));
        assert!(server.session.shutdown_requested);

        let exit = server
            .handle_notification(JsonRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "exit".to_string(),
                params: None,
            })
            .unwrap();
        assert!(exit.is_empty());
    }

    #[test]
    fn lsp_server_tracks_open_change_and_close_document_lifecycle() {
        let (root, uri) = sample_package_root("lifecycle");
        let mut server = EditorLspServer::new(EditorConfig::default());

        let open = open_document(
            &mut server,
            uri.clone(),
            "fun[] main(): int = {\n    return 0\n}\n",
        );
        assert_eq!(server.session.documents.len(), 1);
        assert_eq!(server.session.mappings.len(), 1);
        assert!(open[0].diagnostics.is_empty());

        let changed = server
            .handle_notification(JsonRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "textDocument/didChange".to_string(),
                params: Some(serde_json::to_value(LspDidChangeTextDocumentParams {
                    text_document: LspVersionedTextDocumentIdentifier {
                        uri: uri.clone(),
                        version: 2,
                    },
                    content_changes: vec![LspTextDocumentContentChangeEvent {
                        text: "fun[] main(): int = {\n    return 7\n}\n".to_string(),
                    }],
                }).unwrap()),
            })
            .unwrap();
        assert_eq!(server.session.documents.get(&crate::EditorDocumentUri::parse(&uri).unwrap()).unwrap().version, 2);
        assert!(changed[0].diagnostics.is_empty());

        let closed = server
            .handle_notification(JsonRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "textDocument/didClose".to_string(),
                params: Some(serde_json::to_value(LspDidCloseTextDocumentParams {
                    text_document: LspVersionedTextDocumentIdentifier {
                        uri: uri.clone(),
                        version: 2,
                    },
                }).unwrap()),
            })
            .unwrap();
        assert!(server.session.documents.is_empty());
        assert!(server.session.mappings.is_empty());
        assert!(closed[0].diagnostics.is_empty());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn lsp_server_maps_document_roots_and_surfaces_resolver_diagnostics() {
        let (root, uri) = sample_package_root("resolver_diag");
        fs::write(
            root.join("src/main.fol"),
            "fun[] main(): int = {\n    return missing_value\n}\n",
        )
        .unwrap();
        let mut server = EditorLspServer::new(EditorConfig::default());
        let diagnostics = open_document(
            &mut server,
            uri,
            "fun[] main(): int = {\n    return missing_value\n}\n",
        );

        assert_eq!(server.session.mappings.len(), 1);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].diagnostics[0].code, "R1003");

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn lsp_server_surfaces_parser_diagnostics_from_open_documents() {
        let (root, uri) = sample_package_root("parser_diag");
        let mut server = EditorLspServer::new(EditorConfig::default());
        let diagnostics = open_document(
            &mut server,
            uri,
            "fun[] main(: int = {\n    return 0\n}\n",
        );

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].diagnostics[0].code, "P1001");

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn lsp_server_surfaces_package_loading_diagnostics_from_open_documents() {
        let (root, uri) = sample_package_root("package_diag");
        fs::remove_file(root.join("build.fol")).unwrap();
        let mut server = EditorLspServer::new(EditorConfig::default());
        let diagnostics = open_document(
            &mut server,
            uri,
            "fun[] main(): int = {\n    return 0\n}\n",
        );

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].diagnostics[0].code, "K1001");

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn lsp_server_does_not_report_formal_package_root_errors_for_open_entry_packages() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../..")
            .join("xtra/logtiny");
        let file = root.join("src/lib.fol");
        let uri = format!("file://{}", file.display());
        let text = fs::read_to_string(&file).unwrap();
        let mut server = EditorLspServer::new(EditorConfig::default());
        let diagnostics = open_document(&mut server, uri, &text);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0]
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "K1001"));
    }

    #[test]
    fn lsp_server_filters_build_file_diagnostics_out_of_source_buffers() {
        let root = temp_root("build_diag_filter");
        let src = root.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
        fs::write(root.join("build.fol"), "def root: loc = \"src\"\n").unwrap();
        let file = src.join("main.fol");
        fs::write(&file, "fun[] main(): int = {\n    return 0\n}\n").unwrap();
        let uri = format!("file://{}", file.display());
        let text = fs::read_to_string(&file).unwrap();
        let mut server = EditorLspServer::new(EditorConfig::default());
        let diagnostics = open_document(&mut server, uri, &text);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].diagnostics.is_empty());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn lsp_server_surfaces_typecheck_diagnostics_from_open_documents() {
        let (root, uri) = sample_package_root("typecheck_diag");
        let mut server = EditorLspServer::new(EditorConfig::default());
        let diagnostics = open_document(
            &mut server,
            uri,
            "fun[] main(): int = {\n    return \"nope\"\n}\n",
        );

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].diagnostics[0].code.starts_with('T'));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn lsp_server_handles_hover_definition_and_document_symbols() {
        let (root, uri) = sample_package_root("nav");
        fs::write(
            root.join("src/main.fol"),
            "fun[] helper(): int = {\n    return 7\n}\n\nfun[] main(): int = {\n    return helper()\n}\n",
        )
        .unwrap();
        let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
        let mut server = EditorLspServer::new(EditorConfig::default());
        open_document(&mut server, uri.clone(), &text);

        let hover = server
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: JsonRpcId::Number(3),
                method: "textDocument/hover".to_string(),
                params: Some(
                    serde_json::to_value(LspHoverParams {
                        text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                        position: LspPosition {
                            line: 4,
                            character: 11,
                        },
                    })
                    .unwrap(),
                ),
            })
            .unwrap()
            .unwrap();
        let hover: Option<LspHover> = serde_json::from_value(hover.result.unwrap()).unwrap();
        assert!(hover.unwrap().contents.contains("helper"));

        let definition = server
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: JsonRpcId::Number(4),
                method: "textDocument/definition".to_string(),
                params: Some(
                    serde_json::to_value(LspDefinitionParams {
                        text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                        position: LspPosition {
                            line: 4,
                            character: 11,
                        },
                    })
                    .unwrap(),
                ),
            })
            .unwrap()
            .unwrap();
        let definition: Option<LspLocation> =
            serde_json::from_value(definition.result.unwrap()).unwrap();
        assert_eq!(definition.unwrap().range.start.line, 0);

        let symbols = server
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: JsonRpcId::Number(5),
                method: "textDocument/documentSymbol".to_string(),
                params: Some(
                    serde_json::to_value(LspDocumentSymbolParams {
                        text_document: LspTextDocumentIdentifier { uri },
                    })
                    .unwrap(),
                ),
            })
            .unwrap()
            .unwrap();
        let symbols: Vec<crate::LspDocumentSymbol> =
            serde_json::from_value(symbols.result.unwrap()).unwrap();
        assert!(symbols.iter().any(|symbol| symbol.name == "helper"));
        assert!(symbols.iter().any(|symbol| symbol.name == "main"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn lsp_server_handles_completion_requests() {
        let (root, uri) = sample_package_root("completion_request");
        fs::write(
            root.join("src/main.fol"),
            "fun[] main(): int = {\n    var value: int = 7\n    return value\n}\n",
        )
        .unwrap();
        let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
        let mut server = EditorLspServer::new(EditorConfig::default());
        open_document(&mut server, uri.clone(), &text);

        let completion = server
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: JsonRpcId::Number(30),
                method: "textDocument/completion".to_string(),
                params: Some(
                    serde_json::to_value(LspCompletionParams {
                        text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                        position: LspPosition {
                            line: 2,
                            character: 12,
                        },
                        context: None,
                    })
                    .unwrap(),
                ),
            })
            .unwrap()
            .unwrap();

        let completion: LspCompletionList = serde_json::from_value(completion.result.unwrap()).unwrap();
        assert!(!completion.is_incomplete);
        assert_eq!(completion.items.len(), 1);
        assert_eq!(completion.items[0].label, "value");
        assert_eq!(completion.items[0].detail.as_deref(), Some("binding"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn lsp_server_keeps_nested_document_symbols_stable() {
        let (root, uri) = sample_package_root("nested_symbols");
        fs::write(
            root.join("src/main.fol"),
            "fun[] main(): int = {\n    fun inner(): int = {\n        return 7\n    }\n    return inner()\n}\n",
        )
        .unwrap();
        let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
        let mut server = EditorLspServer::new(EditorConfig::default());
        open_document(&mut server, uri.clone(), &text);

        let symbols = server
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: JsonRpcId::Number(50),
                method: "textDocument/documentSymbol".to_string(),
                params: Some(
                    serde_json::to_value(LspDocumentSymbolParams {
                        text_document: LspTextDocumentIdentifier { uri },
                    })
                    .unwrap(),
                ),
            })
            .unwrap()
            .unwrap();
        let symbols: Vec<crate::LspDocumentSymbol> =
            serde_json::from_value(symbols.result.unwrap()).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "main");
        assert_eq!(symbols[0].children.len(), 1);
        assert_eq!(symbols[0].children[0].name, "inner");

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn lsp_server_resolves_imported_symbol_definitions_and_namespace_symbols() {
        let (root, uri) = sample_loc_workspace_root("import_nav");
        let text = fs::read_to_string(root.join("app/src/main.fol")).unwrap();
        let mut server = EditorLspServer::new(EditorConfig::default());
        open_document(&mut server, uri.clone(), &text);

        let definition = server
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: JsonRpcId::Number(6),
                method: "textDocument/definition".to_string(),
                params: Some(
                    serde_json::to_value(LspDefinitionParams {
                        text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                        position: LspPosition {
                            line: 3,
                            character: 18,
                        },
                    })
                    .unwrap(),
                ),
            })
            .unwrap()
            .unwrap();
        let definition: Option<LspLocation> =
            serde_json::from_value(definition.result.unwrap()).unwrap();
        let definition = definition.unwrap();
        assert!(definition.uri.ends_with("/shared/src/lib.fol"));
        assert_eq!(definition.range.start.line, 0);

        let symbols = server
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: JsonRpcId::Number(7),
                method: "textDocument/documentSymbol".to_string(),
                params: Some(
                    serde_json::to_value(LspDocumentSymbolParams {
                        text_document: LspTextDocumentIdentifier { uri },
                    })
                    .unwrap(),
                ),
            })
            .unwrap()
            .unwrap();
        let symbols: Vec<crate::LspDocumentSymbol> =
            serde_json::from_value(symbols.result.unwrap()).unwrap();
        assert!(symbols.iter().any(|symbol| symbol.name == "shared"));
        assert!(symbols.iter().any(|symbol| symbol.name == "main"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn lsp_server_handles_real_checked_in_package_fixture() {
        let path = PathBuf::from("xtra/logtiny/src/log.fol")
            .canonicalize()
            .expect("checked-in package fixture should canonicalize");
        let uri = format!("file://{}", path.display());
        let text = fs::read_to_string(&path).unwrap();
        let mut server = EditorLspServer::new(EditorConfig::default());
        let diagnostics = open_document(&mut server, uri.clone(), &text);

        assert!(diagnostics[0].diagnostics.is_empty());

        let symbols = server
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: JsonRpcId::Number(8),
                method: "textDocument/documentSymbol".to_string(),
                params: Some(
                    serde_json::to_value(LspDocumentSymbolParams {
                        text_document: LspTextDocumentIdentifier { uri },
                    })
                    .unwrap(),
                ),
            })
            .unwrap()
            .unwrap();
        let symbols: Vec<crate::LspDocumentSymbol> =
            serde_json::from_value(symbols.result.unwrap()).unwrap();
        assert!(symbols.iter().any(|symbol| symbol.name == "Logger"));
        assert!(symbols.iter().any(|symbol| symbol.name == "emit"));
        assert!(symbols.iter().any(|symbol| symbol.name == "DEFAULT"));
    }

    #[test]
    fn lsp_server_surfaces_future_version_boundary_diagnostics() {
        let (root, uri) = sample_package_root("future_boundary");
        let text = "typ Shape(geo): rec[] = {\n    size: int\n}\n\nfun[] main(): int = {\n    return 0\n}\n";
        fs::write(root.join("src/main.fol"), text).unwrap();
        let mut server = EditorLspServer::new(EditorConfig::default());
        let diagnostics = open_document(&mut server, uri, text);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].diagnostics[0].code.starts_with('T'));
        assert!(
            diagnostics[0].diagnostics[0]
                .message
                .contains("V2")
                || diagnostics[0].diagnostics[0]
                    .related_information
                    .iter()
                    .any(|info| info.message.contains("V2"))
        );

        fs::remove_dir_all(root).ok();
    }
}
