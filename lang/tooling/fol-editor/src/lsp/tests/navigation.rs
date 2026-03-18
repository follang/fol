use super::helpers::{open_document, sample_loc_workspace_root, sample_package_root};
use super::super::{
    completion_helpers::completion_context, completion_helpers::CompletionContext,
    EditorLspServer, JsonRpcId, JsonRpcNotification, JsonRpcRequest, LspCompletionContext,
    LspCompletionList, LspCompletionParams, LspDefinitionParams,
    LspDidChangeTextDocumentParams, LspDidCloseTextDocumentParams,
    LspDidOpenTextDocumentParams, LspDocumentSymbolParams, LspHover, LspHoverParams,
    LspInitializeResult, LspLocation, LspPosition, LspPublishDiagnosticsParams,
    LspTextDocumentContentChangeEvent, LspTextDocumentIdentifier, LspTextDocumentItem,
    LspVersionedTextDocumentIdentifier,
};
use crate::{EditorConfig, EditorDocument, EditorDocumentUri};
use std::fs;
use std::path::PathBuf;

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
        diagnostics[0].diagnostics[0].message.contains("V2")
            || diagnostics[0].diagnostics[0]
                .related_information
                .iter()
                .any(|info| info.message.contains("V2"))
    );

    fs::remove_dir_all(root).ok();
}
