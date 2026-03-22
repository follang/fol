use super::super::{
    EditorLspServer, JsonRpcNotification, LspDidOpenTextDocumentParams, LspPublishDiagnosticsParams,
    LspTextDocumentItem,
};
use std::fs;
use std::path::PathBuf;

pub(super) fn temp_root(label: &str) -> PathBuf {
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

pub(super) fn sample_package_root(label: &str) -> (PathBuf, String) {
    let root = temp_root(label);
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
    fs::write(
        root.join("build.fol"),
        "pro[] build(graph: Graph): non = {\n    return graph;\n};\n",
    )
    .unwrap();
    let file = src.join("main.fol");
    fs::write(&file, "fun[] main(): int = {\n    return 0;\n};\n").unwrap();
    let uri = format!("file://{}", file.display());
    (root, uri)
}

pub(super) fn sample_loc_workspace_root(label: &str) -> (PathBuf, String) {
    let root = temp_root(label);
    let app_src = root.join("app/src");
    let shared_src = root.join("shared/src");
    fs::create_dir_all(&app_src).unwrap();
    fs::create_dir_all(&shared_src).unwrap();

    fs::write(root.join("app/package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();
    fs::write(
        root.join("app/build.fol"),
        "pro[] build(graph: Graph): non = {\n    return graph;\n};\n",
    )
    .unwrap();
    fs::write(
        root.join("shared/package.yaml"),
        "name: shared\nversion: 0.1.0\n",
    )
    .unwrap();
    fs::write(
        root.join("shared/build.fol"),
        "pro[] build(graph: Graph): non = {\n    return graph;\n};\n",
    )
    .unwrap();

    fs::write(
        root.join("app/src/main.fol"),
        "use shared: loc = {\"../shared\"};\n\nfun[] main(): int = {\n    return shared.helper();\n};\n",
    )
    .unwrap();
    fs::write(
        root.join("shared/src/lib.fol"),
        "fun[exp] helper(): int = {\n    return 9;\n};\n",
    )
    .unwrap();

    let uri = format!("file://{}", root.join("app/src/main.fol").display());
    (root, uri)
}

pub(super) fn open_document(
    server: &mut EditorLspServer,
    uri: String,
    text: &str,
) -> Vec<LspPublishDiagnosticsParams> {
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
