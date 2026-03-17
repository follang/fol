use crate::{
    diagnostic_to_lsp, map_document_workspace, materialize_analysis_overlay, EditorConfig,
    EditorDocument, EditorDocumentUri, EditorError, EditorErrorKind, EditorResult, EditorSession,
    EditorWorkspaceMapping, LspDiagnostic,
};
use fol_diagnostics::ToDiagnostic;
use fol_package::{PackageError, PackageSession, PackageSourceKind};
use fol_parser::ast::{AstParser, ParseError};
use fol_resolver::{Resolver, ResolverError};
use fol_stream::{FileStream, Source, SourceType};
use serde::{Deserialize, Serialize};
use std::path::Path;

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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LspServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LspTextDocumentSyncOptions {
    pub open_close: bool,
    pub change: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LspServerCapabilities {
    pub text_document_sync: LspTextDocumentSyncOptions,
    pub hover_provider: bool,
    pub definition_provider: bool,
    pub document_symbol_provider: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LspInitializeParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LspInitializeResult {
    pub capabilities: LspServerCapabilities,
    pub server_info: LspServerInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LspTextDocumentItem {
    pub uri: String,
    pub language_id: String,
    pub version: i32,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LspVersionedTextDocumentIdentifier {
    pub uri: String,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LspTextDocumentContentChangeEvent {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LspDidOpenTextDocumentParams {
    pub text_document: LspTextDocumentItem,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LspDidChangeTextDocumentParams {
    pub text_document: LspVersionedTextDocumentIdentifier,
    pub content_changes: Vec<LspTextDocumentContentChangeEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LspDidCloseTextDocumentParams {
    pub text_document: LspVersionedTextDocumentIdentifier,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LspPublishDiagnosticsParams {
    pub uri: String,
    pub diagnostics: Vec<LspDiagnostic>,
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
                        hover_provider: false,
                        definition_provider: false,
                        document_symbol_provider: false,
                    },
                    server_info: LspServerInfo {
                        name: "fol-editor".to_string(),
                        version: env!("CARGO_PKG_VERSION").to_string(),
                    },
                })
                .expect("initialize result should serialize")),
            })),
            "shutdown" => {
                self.session.shutdown_requested = true;
                Ok(Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::Value::Null),
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
        let document = self
            .session
            .documents
            .get(uri)
            .ok_or_else(|| {
                EditorError::new(
                    EditorErrorKind::DocumentNotOpen,
                    format!("document '{}' is not open", uri.as_str()),
                )
            })?;
        let mapping = self
            .session
            .mappings
            .get(uri.as_str())
            .cloned()
            .unwrap_or(map_document_workspace(document.path.as_path(), &self.session.config)?);
        let diagnostics = analyze_document(document, &mapping)?;
        Ok(LspPublishDiagnosticsParams {
            uri: uri.as_str().to_string(),
            diagnostics,
        })
    }
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
    let overlay = materialize_analysis_overlay(mapping, document)?;
    if let Some(package_root) = overlay.package_root() {
        let parser_diags = parse_directory_diagnostics(package_root)?;
        if !parser_diags.is_empty() {
            return Ok(parser_diags);
        }

        let mut package_session = PackageSession::new();
        let prepared = match package_session.load_directory_package(package_root, PackageSourceKind::Local) {
            Ok(prepared) => prepared,
            Err(error) => return Ok(vec![diagnostic_to_lsp(&error.to_diagnostic())]),
        };

        let mut resolver = Resolver::new();
        match resolver.resolve_prepared_workspace(prepared) {
            Ok(_) => Ok(Vec::new()),
            Err(errors) => Ok(errors
                .iter()
                .map(|error| diagnostic_to_lsp(&error.to_diagnostic()))
                .collect()),
        }
    } else {
        Ok(parse_single_file_diagnostics(&mapping.document_path, &document.text)?)
    }
}

fn parse_single_file_diagnostics(path: &Path, text: &str) -> EditorResult<Vec<LspDiagnostic>> {
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

fn parse_directory_diagnostics(root: &Path) -> EditorResult<Vec<LspDiagnostic>> {
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
            .map(|error| glitch_to_lsp(error.as_ref()))
            .collect()),
    }
}

fn glitch_to_lsp(error: &dyn fol_types::Glitch) -> LspDiagnostic {
    if let Some(parse_error) = error.as_any().downcast_ref::<ParseError>() {
        return diagnostic_to_lsp(&parse_error.to_diagnostic());
    }
    if let Some(package_error) = error.as_any().downcast_ref::<PackageError>() {
        return diagnostic_to_lsp(&package_error.to_diagnostic());
    }
    if let Some(resolver_error) = error.as_any().downcast_ref::<ResolverError>() {
        return diagnostic_to_lsp(&resolver_error.to_diagnostic());
    }
    diagnostic_to_lsp(&fol_diagnostics::Diagnostic::error("E9999", error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{
        EditorLspServer, JsonRpcId, JsonRpcNotification, JsonRpcRequest,
        LspDidChangeTextDocumentParams, LspDidCloseTextDocumentParams, LspDidOpenTextDocumentParams,
        LspInitializeResult, LspTextDocumentContentChangeEvent, LspTextDocumentItem,
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
}
