use crate::{
    location_to_range, EditorDocument, EditorError, EditorErrorKind, EditorResult, LspDiagnostic,
    LspLocation, LspPosition, LspRange, LspTextEdit, LspWorkspaceEdit,
};
use fol_intrinsics::{
    intrinsic_registry, IntrinsicAvailability, IntrinsicStatus, IntrinsicSurface,
};
use fol_parser::ast::{AstNode, SyntaxNodeId};
use std::path::PathBuf;

use super::completion_helpers::{
    completion_builtin_type_item,
    completion_intrinsic_item, completion_item_from_symbol, completion_namespace_item,
    completion_symbol_is_plain_top_level_candidate, completion_symbol_is_root_visible,
    current_routine_name, dedupe_completion_items, fallback_decl_name,
    fallback_items_from_package_dir, position_to_offset, render_checked_type, render_symbol_kind,
    symbol_kind_code, symbol_visibility_matches_namespace_root, CompletionContext,
};
use super::types::{
    EditorCompletionItem, LspDocumentSymbol, LspHover, LspParameterInformation, LspSignatureHelp,
    LspSignatureInformation,
};

const SEMANTIC_TOKEN_TYPES: &[&str] = &["namespace", "type", "function", "parameter", "variable"];

pub(super) fn semantic_token_types() -> &'static [&'static str] {
    SEMANTIC_TOKEN_TYPES
}

#[derive(Debug)]
pub(crate) struct SemanticSnapshot {
    pub(super) analyzed_path: Option<PathBuf>,
    pub(super) source_document_path: PathBuf,
    pub(super) source_package_root: Option<PathBuf>,
    pub(super) diagnostics: Vec<LspDiagnostic>,
    pub(super) resolved_workspace: Option<fol_resolver::ResolvedWorkspace>,
    pub(super) typed_workspace: Option<fol_typecheck::TypedWorkspace>,
}

impl SemanticSnapshot {
    pub(super) fn signature_help(
        &self,
        document: &EditorDocument,
        position: LspPosition,
    ) -> Option<LspSignatureHelp> {
        let resolved = self.resolved_workspace.as_ref()?;
        let typed = self.typed_workspace.as_ref()?;
        let (package, program) = self.current_resolved_package()?;
        let typed_package = typed.package(&package.identity)?;
        let cursor_offset = offset_for_position(&document.text, position)?;
        let call_site = self.call_site_at_position(program, document, cursor_offset)?;
        let reference = program
            .all_references()
            .find(|reference| reference.syntax_id == Some(call_site.callee_syntax_id))?;
        let symbol_id = reference.resolved?;
        let declared_type = typed_package.program.typed_symbol(symbol_id)?.declared_type?;
        let signature = match typed_package.program.type_table().get(declared_type) {
            Some(fol_typecheck::CheckedType::Routine(signature)) => signature,
            _ => return None,
        };
        let parameters = signature
            .params
            .iter()
            .map(|type_id| render_checked_type(typed_package.program.type_table(), *type_id))
            .collect::<Vec<_>>();
        let label = render_signature_label(
            program.symbol(symbol_id).map(|symbol| symbol.name.as_str()).unwrap_or(&call_site.display_name),
            &parameters,
            signature.return_type.map(|type_id| {
                render_checked_type(typed_package.program.type_table(), type_id)
            }),
            signature.error_type.map(|type_id| {
                render_checked_type(typed_package.program.type_table(), type_id)
            }),
        );
        let active_parameter = if parameters.is_empty() {
            None
        } else {
            Some(call_site.active_parameter.min(parameters.len().saturating_sub(1)) as u32)
        };

        let _ = resolved;
        Some(LspSignatureHelp {
            signatures: vec![LspSignatureInformation {
                label,
                parameters: parameters
                    .into_iter()
                    .map(|label| LspParameterInformation { label })
                    .collect(),
            }],
            active_signature: Some(0),
            active_parameter,
        })
    }

    pub(super) fn semantic_tokens_for_current_path(&self) -> Vec<u32> {
        let Some(program) = self.current_program() else {
            return Vec::new();
        };
        let Some(analyzed_path) = self.analyzed_path.as_ref() else {
            return Vec::new();
        };
        let path_text = analyzed_path.to_string_lossy();
        let mut entries = std::collections::BTreeSet::new();

        for symbol in program.all_symbols() {
            let Some(origin) = symbol.origin.as_ref() else { continue };
            let Some(file) = origin.file.as_ref() else { continue };
            if file != &path_text {
                continue;
            }
            let Some(token_type) = semantic_token_type_for_symbol_kind(symbol.kind) else {
                continue;
            };
            let line = origin.line.saturating_sub(1) as u32;
            let start = origin.column.saturating_sub(1) as u32;
            let length = origin.length as u32;
            if length == 0 {
                continue;
            }
            entries.insert((line, start, length, token_type, 0_u32));
        }

        for reference in program.all_references() {
            let Some(symbol_id) = reference.resolved else {
                continue;
            };
            let Some(symbol) = program.symbol(symbol_id) else {
                continue;
            };
            let Some(token_type) = semantic_token_type_for_symbol_kind(symbol.kind) else {
                continue;
            };
            let Some(syntax_id) = reference.syntax_id else { continue };
            let Some(origin) = program.syntax_index().origin(syntax_id) else {
                continue;
            };
            let Some(file) = origin.file.as_ref() else { continue };
            if file != &path_text {
                continue;
            }
            let line = origin.line.saturating_sub(1) as u32;
            let start = origin.column.saturating_sub(1) as u32;
            let length = origin.length as u32;
            if length == 0 {
                continue;
            }
            entries.insert((line, start, length, token_type, 0_u32));
        }

        let mut data = Vec::with_capacity(entries.len() * 5);
        let mut previous_line = 0_u32;
        let mut previous_start = 0_u32;
        for (index, (line, start, length, token_type, modifiers)) in entries.into_iter().enumerate()
        {
            let delta_line = if index == 0 { line } else { line - previous_line };
            let delta_start = if index == 0 || delta_line != 0 {
                start
            } else {
                start - previous_start
            };
            data.extend([delta_line, delta_start, length, token_type, modifiers]);
            previous_line = line;
            previous_start = start;
        }
        data
    }

    pub(super) fn completion_items(
        &self,
        document: &EditorDocument,
        position: LspPosition,
        context: CompletionContext,
    ) -> Vec<EditorCompletionItem> {
        if self.current_program().is_none() {
            return self.fallback_completion_items(document, position, context);
        }
        match context {
            CompletionContext::Plain => {}
            CompletionContext::TypePosition => {
                let mut items = self.builtin_type_completion_items();
                items.extend(self.visible_named_type_completion_items());
                return dedupe_completion_items(items);
            }
            CompletionContext::QualifiedPath { qualifier } => {
                return self.qualified_completion_items(&qualifier);
            }
            CompletionContext::DotTrigger => return self.dot_intrinsic_fallback_completion_items(),
        }
        let mut items = self.local_completion_items(position);
        items.extend(self.current_package_top_level_completion_items());
        items.extend(self.import_alias_completion_items(position));
        dedupe_completion_items(items)
    }

    fn fallback_completion_items(
        &self,
        document: &EditorDocument,
        position: LspPosition,
        context: CompletionContext,
    ) -> Vec<EditorCompletionItem> {
        match context {
            CompletionContext::DotTrigger => self.dot_intrinsic_fallback_completion_items(),
            CompletionContext::QualifiedPath { qualifier } => {
                self.fallback_qualified_completion_items(&qualifier)
            }
            CompletionContext::TypePosition => {
                let mut items = self.builtin_type_completion_items();
                items.extend(self.fallback_local_named_type_items(document));
                items.extend(self.fallback_imported_named_type_items(document));
                dedupe_completion_items(items)
            }
            CompletionContext::Plain => {
                if position_to_offset(&document.text, position).is_none() {
                    if let Some(line) = document.text.lines().nth(position.line as usize) {
                        if line.contains("::") {
                            let aliases = self.fallback_import_alias_items(document);
                            if aliases.len() == 1 {
                                let items = self.fallback_imported_package_items(&aliases[0].label);
                                if !items.is_empty() {
                                    return dedupe_completion_items(items);
                                }
                            }
                        }
                    }
                }
                let mut items = self.fallback_local_scope_items(document, position);
                items.extend(self.fallback_current_package_top_level_items(document, position));
                items.extend(self.fallback_import_alias_items(document));
                dedupe_completion_items(items)
            }
        }
    }

    fn builtin_type_completion_items(&self) -> Vec<EditorCompletionItem> {
        fol_typecheck::BuiltinType::ALL_NAMES
            .iter()
            .map(|name| completion_builtin_type_item(name))
            .collect()
    }

    // COMPILER-BACKED: reads from resolved all_symbols
    fn visible_named_type_completion_items(&self) -> Vec<EditorCompletionItem> {
        let Some(program) = self.current_program() else {
            return Vec::new();
        };
        program
            .all_symbols()
            .filter(|symbol| {
                matches!(
                    symbol.kind,
                    fol_resolver::SymbolKind::Type | fol_resolver::SymbolKind::Alias
                )
            })
            .filter(|symbol| completion_symbol_is_root_visible(program, symbol))
            .map(completion_item_from_symbol)
            .collect()
    }

    // COMPILER-BACKED: reads from resolver namespace/scope + child namespaces
    fn qualified_completion_items(&self, qualifier: &str) -> Vec<EditorCompletionItem> {
        let Some(program) = self.current_program() else {
            return Vec::new();
        };
        let qualifier_root = qualifier.split("::").next().unwrap_or(qualifier);
        let imported_root = program.all_symbols().any(|symbol| {
            symbol.kind == fol_resolver::SymbolKind::ImportAlias && symbol.name == qualifier_root
        });
        let mut items = Vec::new();

        if let Some(scope_id) = program.namespace_scope(qualifier) {
            items.extend(
                program
                    .symbols_in_scope(scope_id)
                    .into_iter()
                    .filter(|symbol| {
                        symbol_visibility_matches_namespace_root(symbol, imported_root)
                    })
                    .map(completion_item_from_symbol),
            );
        }

        for source_unit in program.source_units.iter() {
            if source_unit.namespace != qualifier {
                continue;
            }
            items.extend(
                program
                    .symbols_in_scope(source_unit.scope_id)
                    .into_iter()
                    .filter(|symbol| {
                        symbol_visibility_matches_namespace_root(symbol, imported_root)
                    })
                    .map(completion_item_from_symbol),
            );
        }

        let prefix = format!("{qualifier}::");
        let mut child_namespaces = std::collections::BTreeSet::new();
        for (_, scope) in program.scopes.iter_with_ids() {
            let fol_resolver::ScopeKind::NamespaceRoot { namespace } = &scope.kind else {
                continue;
            };
            let Some(remainder) = namespace.strip_prefix(&prefix) else {
                continue;
            };
            let child = remainder.split("::").next().unwrap_or("");
            if !child.is_empty() {
                child_namespaces.insert(child.to_string());
            }
        }
        items.extend(child_namespaces.into_iter().map(completion_namespace_item));

        dedupe_completion_items(items)
    }

    // COMPILER-BACKED: intrinsic registry is the canonical source
    fn dot_intrinsic_fallback_completion_items(&self) -> Vec<EditorCompletionItem> {
        intrinsic_registry()
            .iter()
            .filter(|entry| entry.surface == IntrinsicSurface::DotRootCall)
            .filter(|entry| entry.availability == IntrinsicAvailability::V1)
            .filter(|entry| entry.status == IntrinsicStatus::Implemented)
            .map(completion_intrinsic_item)
            .collect()
    }

    // COMPILER-BACKED: reads from resolver scope chain
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
                        | fol_resolver::SymbolKind::Parameter
                        | fol_resolver::SymbolKind::GenericParameter
                        | fol_resolver::SymbolKind::LoopBinder
                        | fol_resolver::SymbolKind::RollingBinder
                        | fol_resolver::SymbolKind::Capture
                ) {
                    continue;
                }
                items.push(completion_item_from_symbol(symbol));
            }
            cursor = program
                .scope(current_scope_id)
                .and_then(|scope| scope.parent);
        }
        items
    }

    // COMPILER-BACKED: reads from resolved program namespace/source-unit scopes
    fn current_package_top_level_completion_items(
        &self,
    ) -> Vec<EditorCompletionItem> {
        let Some(program) = self.current_program() else {
            return Vec::new();
        };
        let Some(namespace) = self.current_namespace() else {
            return Vec::new();
        };
        let mut items = Vec::new();
        if let Some(scope_id) = program.namespace_scope(namespace.as_str()) {
            items.extend(
                program
                    .symbols_in_scope(scope_id)
                    .into_iter()
                    .filter(|symbol| symbol.mounted_from.is_none())
                    .filter(|symbol| {
                        completion_symbol_is_plain_top_level_candidate(program, symbol)
                    })
                    .map(completion_item_from_symbol),
            );
        }
        for source_unit in program
            .source_units
            .iter()
            .filter(|unit| unit.namespace == namespace)
        {
            items.extend(
                program
                    .symbols_in_scope(source_unit.scope_id)
                    .into_iter()
                    .filter(|symbol| symbol.mounted_from.is_none())
                    .filter(|symbol| {
                        completion_symbol_is_plain_top_level_candidate(program, symbol)
                    })
                    .map(completion_item_from_symbol),
            );
        }
        items
    }

    // COMPILER-BACKED: reads from resolver scope chain
    fn import_alias_completion_items(&self, position: LspPosition) -> Vec<EditorCompletionItem> {
        let Some((program, scope_id)) = self.scope_at_position(position) else {
            return Vec::new();
        };
        let mut items = Vec::new();
        let mut cursor = Some(scope_id);
        while let Some(current_scope_id) = cursor {
            for symbol in program.symbols_in_scope(current_scope_id) {
                if symbol.kind != fol_resolver::SymbolKind::ImportAlias {
                    continue;
                }
                items.push(completion_item_from_symbol(symbol));
            }
            cursor = program
                .scope(current_scope_id)
                .and_then(|scope| scope.parent);
        }
        items
    }

    // FALLBACK: text-scans for `var ` bindings and `fun` parameters when
    // resolver data is absent or incomplete. Required for broken documents.
    fn fallback_local_scope_items(
        &self,
        document: &EditorDocument,
        position: LspPosition,
    ) -> Vec<EditorCompletionItem> {
        let offset = position_to_offset(&document.text, position).unwrap_or(document.text.len());
        let before_cursor = &document.text[..offset];
        let mut items = self.fallback_import_alias_items(document);
        items.extend(self.fallback_current_package_top_level_items(document, position));

        if let Some(header) = before_cursor
            .rmatch_indices("fun")
            .next()
            .map(|(index, _)| &before_cursor[index..])
        {
            if let Some(open) = header.find('(') {
                if let Some(close) = header[open + 1..].find(')') {
                    for param in header[open + 1..open + 1 + close].split(',') {
                        let name = param.split(':').next().unwrap_or("").trim();
                        if !name.is_empty() {
                            items.push(EditorCompletionItem {
                                label: name.to_string(),
                                kind: 6,
                                detail: Some("parameter".to_string()),
                                insert_text: None,
                            });
                        }
                    }
                }
            }
        }

        for line in document.text.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("var ") {
                let name = rest
                    .split(|ch: char| ch == ':' || ch == '=' || ch.is_whitespace())
                    .next()
                    .unwrap_or("")
                    .trim();
                if !name.is_empty() {
                    items.push(EditorCompletionItem {
                        label: name.to_string(),
                        kind: 6,
                        detail: Some("binding".to_string()),
                        insert_text: None,
                    });
                }
            }
        }

        items
    }

    // FALLBACK: text-matches `fun[`, `pro[`, `typ[`, `ali[`, `def[` prefixes
    // when resolver data is absent. Required for broken documents.
    fn fallback_current_package_top_level_items(
        &self,
        document: &EditorDocument,
        position: LspPosition,
    ) -> Vec<EditorCompletionItem> {
        let mut items = Vec::new();
        let current_routine = current_routine_name(&document.text, position);
        for line in document.text.lines() {
            let trimmed = line.trim();
            if let Some(name) = fallback_decl_name(
                trimmed,
                &["fun[] ", "fun[", "log[] ", "log[", "pro[] ", "pro["],
            ) {
                if current_routine.as_deref() == Some(name.as_str()) {
                    continue;
                }
                items.push(EditorCompletionItem {
                    label: name,
                    kind: 3,
                    detail: Some("routine".to_string()),
                    insert_text: None,
                });
            } else if let Some(name) = fallback_decl_name(trimmed, &["def[] ", "def["]) {
                items.push(EditorCompletionItem {
                    label: name,
                    kind: 12,
                    detail: Some("definition".to_string()),
                    insert_text: None,
                });
            } else if let Some(name) = fallback_decl_name(trimmed, &["typ[] ", "typ[", "typ "]) {
                items.push(EditorCompletionItem {
                    label: name,
                    kind: 22,
                    detail: Some("type".to_string()),
                    insert_text: None,
                });
            } else if let Some(name) = fallback_decl_name(trimmed, &["ali[] ", "ali[", "ali "]) {
                items.push(EditorCompletionItem {
                    label: name,
                    kind: 22,
                    detail: Some("type alias".to_string()),
                    insert_text: None,
                });
            }
        }
        items
    }

    // FALLBACK: filters top-level text fallback for type/alias items only
    fn fallback_local_named_type_items(
        &self,
        document: &EditorDocument,
    ) -> Vec<EditorCompletionItem> {
        self.fallback_current_package_top_level_items(
            document,
            LspPosition {
                line: u32::MAX,
                character: u32::MAX,
            },
        )
        .into_iter()
        .filter(|item| {
            item.detail.as_deref() == Some("type") || item.detail.as_deref() == Some("type alias")
        })
        .collect()
    }

    // FALLBACK: text-matches `use ` prefix to find import aliases
    fn fallback_import_alias_items(&self, document: &EditorDocument) -> Vec<EditorCompletionItem> {
        document
            .text
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                let rest = trimmed.strip_prefix("use ")?;
                let alias = rest.split(':').next()?.trim();
                (!alias.is_empty()).then(|| EditorCompletionItem {
                    label: alias.to_string(),
                    kind: 9,
                    detail: Some("namespace".to_string()),
                    insert_text: None,
                })
            })
            .collect()
    }

    // FALLBACK: reads imported package files from disk + text-scans
    fn fallback_imported_named_type_items(
        &self,
        document: &EditorDocument,
    ) -> Vec<EditorCompletionItem> {
        let aliases = self.fallback_import_alias_items(document);
        let mut items = Vec::new();
        for alias in aliases {
            items.extend(
                self.fallback_imported_package_items(&alias.label)
                    .into_iter()
                    .filter(|item| {
                        item.detail.as_deref() == Some("type")
                            || item.detail.as_deref() == Some("type alias")
                    }),
            );
        }
        items
    }

    // FALLBACK: combines local namespace + imported package fallbacks
    fn fallback_qualified_completion_items(&self, qualifier: &str) -> Vec<EditorCompletionItem> {
        let mut items = self.fallback_local_namespace_items(qualifier);
        items.extend(self.fallback_imported_package_items(qualifier));
        dedupe_completion_items(items)
    }

    // FALLBACK: reads imported package files from disk + text-scans declarations
    fn fallback_imported_package_items(&self, qualifier: &str) -> Vec<EditorCompletionItem> {
        let Some(package_root) = &self.source_package_root else {
            return Vec::new();
        };
        let text = std::fs::read_to_string(&self.source_document_path).unwrap_or_default();
        let rel_path = text.lines().find_map(|line| {
            let trimmed = line.trim();
            let rest = trimmed.strip_prefix("use ")?;
            let (alias, rhs) = rest.split_once(':')?;
            (alias.trim() == qualifier).then_some(rhs.trim().to_string())
        });
        let Some(rhs) = rel_path else {
            return Vec::new();
        };
        let Some(start) = rhs.find('"') else {
            return Vec::new();
        };
        let tail = &rhs[start + 1..];
        let Some(end) = tail.find('"') else {
            return Vec::new();
        };
        let target = package_root.join(&tail[..end]);
        fallback_items_from_package_dir(&target)
    }

    // FALLBACK: reads filesystem directories for namespace items
    fn fallback_local_namespace_items(&self, qualifier: &str) -> Vec<EditorCompletionItem> {
        let Some(package_root) = &self.source_package_root else {
            return Vec::new();
        };
        let namespace_dir = package_root.join("src").join(qualifier.replace("::", "/"));
        let mut items = fallback_items_from_package_dir(&namespace_dir);
        if let Ok(entries) = std::fs::read_dir(&namespace_dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
                    if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                        items.push(EditorCompletionItem {
                            label: name.to_string(),
                            kind: 9,
                            detail: Some("namespace".to_string()),
                            insert_text: None,
                        });
                    }
                }
            }
        }
        items
    }

    pub(super) fn current_program(&self) -> Option<&fol_resolver::ResolvedProgram> {
        let resolved = self.resolved_workspace.as_ref()?;
        let analyzed_path = self.analyzed_path.as_ref()?;
        let path_text = analyzed_path.to_string_lossy();
        resolved.packages().find_map(|package| {
            let program = &package.program;
            program
                .source_units
                .iter()
                .any(|unit| unit.path == path_text)
                .then_some(program)
        })
    }

    fn scope_at_position(
        &self,
        position: LspPosition,
    ) -> Option<(&fol_resolver::ResolvedProgram, fol_resolver::ScopeId)> {
        use super::analysis::{nearest_scope_before_position, syntax_at_position};
        let program = self.current_program()?;
        let analyzed_path = self.analyzed_path.as_ref()?;
        if let Some(syntax_id) = syntax_at_position(program, analyzed_path.as_path(), position) {
            if let Some(scope_id) = program.scope_for_syntax(syntax_id) {
                return Some((program, scope_id));
            }
        }
        if let Some(scope_id) =
            nearest_scope_before_position(program, analyzed_path.as_path(), position)
        {
            return Some((program, scope_id));
        }
        self.current_source_unit(program)
            .map(|unit| (program, unit.scope_id))
    }

    fn current_namespace(&self) -> Option<String> {
        let program = self.current_program()?;
        self.current_source_unit(program)
            .map(|unit| unit.namespace.clone())
    }

    fn current_resolved_package(
        &self,
    ) -> Option<(&fol_resolver::ResolvedPackage, &fol_resolver::ResolvedProgram)> {
        let resolved = self.resolved_workspace.as_ref()?;
        let analyzed_path = self.analyzed_path.as_ref()?;
        let path_text = analyzed_path.to_string_lossy();
        resolved.packages().find_map(|package| {
            package
                .program
                .source_units
                .iter()
                .any(|unit| unit.path == path_text)
                .then_some((package, &package.program))
        })
    }

    fn call_site_at_position(
        &self,
        program: &fol_resolver::ResolvedProgram,
        document: &EditorDocument,
        cursor_offset: usize,
    ) -> Option<SignatureCallSite> {
        let source_unit = self.current_source_unit(program)?;
        let path = self.analyzed_path.as_ref()?;
        let text = document.text.as_str();
        let mut best: Option<SignatureCallSite> = None;
        for item in &program.syntax().source_units {
            if item.path != source_unit.path {
                continue;
            }
            for top_level in &item.items {
                visit_call_sites(
                    &top_level.node,
                    program,
                    path.as_path(),
                    text,
                    cursor_offset,
                    &mut best,
                );
            }
            break;
        }
        best
    }

    // COMPILER-BACKED: resolver reference lookup (no text fallback)
    pub(super) fn reference_at(
        &self,
        position: LspPosition,
    ) -> Option<&fol_resolver::ResolvedReference> {
        use super::analysis::syntax_at_position;
        let resolved = self.resolved_workspace.as_ref()?;
        let analyzed_path = self.analyzed_path.as_ref()?;
        let needle = resolved.packages().find_map(|package| {
            let program = &package.program;
            let syntax_id = syntax_at_position(program, analyzed_path.as_path(), position)?;
            program
                .all_references()
                .find(|reference| reference.syntax_id == Some(syntax_id))
        })?;
        Some(needle)
    }

    // COMPILER-BACKED: resolved symbol + typed type (no text fallback)
    pub(super) fn hover_for_reference(
        &self,
        reference: &fol_resolver::ResolvedReference,
    ) -> Option<LspHover> {
        let resolved = self.resolved_workspace.as_ref()?;
        for package in resolved.packages() {
            let program = &package.program;
            if let Some(symbol_id) = reference.resolved {
                let symbol = program.symbol(symbol_id)?;
                let origin = symbol.origin.as_ref()?;
                let type_summary = self
                    .typed_workspace
                    .as_ref()
                    .and_then(|typed| typed.package(&package.identity))
                    .and_then(|typed_package| typed_package.program.typed_symbol(symbol_id))
                    .and_then(|typed_symbol| typed_symbol.declared_type)
                    .map(|type_id| {
                        let typed_package = self
                            .typed_workspace
                            .as_ref()
                            .and_then(|typed| typed.package(&package.identity))
                            .expect("typed package should exist when declared type is available");
                        render_checked_type(typed_package.program.type_table(), type_id)
                    })
                    .unwrap_or_else(|| "unknown".to_string());
                return Some(LspHover {
                    contents: format!(
                        "{} {}: {}",
                        render_symbol_kind(symbol.kind),
                        symbol.name,
                        type_summary
                    ),
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

    // COMPILER-BACKED: resolved symbol origin (no text fallback)
    pub(super) fn definition_for_reference(
        &self,
        reference: &fol_resolver::ResolvedReference,
    ) -> Option<LspLocation> {
        let resolved = self.resolved_workspace.as_ref()?;
        for package in resolved.packages() {
            let program = &package.program;
            if let Some(symbol_id) = reference.resolved {
                let symbol = program.symbol(symbol_id)?;
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

    pub(super) fn references_for_reference(
        &self,
        reference: &fol_resolver::ResolvedReference,
        include_declaration: bool,
    ) -> Vec<LspLocation> {
        let Some(resolved) = self.resolved_workspace.as_ref() else {
            return Vec::new();
        };
        let Some(symbol_id) = reference.resolved else {
            return Vec::new();
        };
        let mut locations = Vec::new();

        for package in resolved.packages() {
            let program = &package.program;
            let Some(symbol) = program.symbol(symbol_id) else {
                continue;
            };

            if include_declaration {
                if let Some(origin) = symbol.origin.as_ref() {
                    if let Some(file) = origin.file.as_ref() {
                        locations.push(LspLocation {
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
            }

            for hit in program.all_references().filter(|hit| hit.resolved == Some(symbol_id)) {
                let Some(syntax_id) = hit.syntax_id else { continue };
                let Some(origin) = program.syntax_index().origin(syntax_id) else {
                    continue;
                };
                let Some(file) = origin.file.as_ref() else { continue };
                locations.push(LspLocation {
                    uri: format!("file://{file}"),
                    range: location_to_range(&fol_diagnostics::DiagnosticLocation {
                        file: Some(file.clone()),
                        line: origin.line,
                        column: origin.column,
                        length: Some(origin.length),
                    }),
                });
            }
            break;
        }

        locations.sort_by(|left, right| {
            left.uri
                .cmp(&right.uri)
                .then(left.range.start.line.cmp(&right.range.start.line))
                .then(left.range.start.character.cmp(&right.range.start.character))
        });
        locations.dedup_by(|left, right| left == right);
        locations
    }

    pub(super) fn rename_for_reference(
        &self,
        reference: &fol_resolver::ResolvedReference,
        new_name: &str,
    ) -> EditorResult<LspWorkspaceEdit> {
        let resolved = self.resolved_workspace.as_ref().ok_or_else(|| {
            EditorError::new(
                EditorErrorKind::InvalidInput,
                "rename requires a resolved workspace",
            )
        })?;
        let symbol_id = reference.resolved.ok_or_else(|| {
            EditorError::new(EditorErrorKind::InvalidInput, "rename target is unresolved")
        })?;
        let analyzed_path = self.analyzed_path.as_ref().ok_or_else(|| {
            EditorError::new(
                EditorErrorKind::InvalidInput,
                "rename target has no analyzed document path",
            )
        })?;
        let analyzed_uri = format!("file://{}", analyzed_path.display());

        for package in resolved.packages() {
            let program = &package.program;
            let Some(symbol) = program.symbol(symbol_id) else {
                continue;
            };

            if !matches!(
                symbol.kind,
                fol_resolver::SymbolKind::ValueBinding
                    | fol_resolver::SymbolKind::LabelBinding
                    | fol_resolver::SymbolKind::DestructureBinding
                    | fol_resolver::SymbolKind::Parameter
                    | fol_resolver::SymbolKind::Capture
                    | fol_resolver::SymbolKind::LoopBinder
                    | fol_resolver::SymbolKind::RollingBinder
            ) {
                return Err(EditorError::new(
                    EditorErrorKind::InvalidInput,
                    format!(
                        "rename currently supports same-file local symbols only, not {}",
                        render_symbol_kind(symbol.kind)
                    ),
                ));
            }

            let mut edits = Vec::new();
            let declaration = symbol.origin.as_ref().ok_or_else(|| {
                EditorError::new(
                    EditorErrorKind::InvalidInput,
                    "rename target is missing a declaration location",
                )
            })?;
            let declaration_file = declaration.file.as_ref().ok_or_else(|| {
                EditorError::new(
                    EditorErrorKind::InvalidInput,
                    "rename target is missing a declaration file",
                )
            })?;
            if declaration_file != &analyzed_path.to_string_lossy() {
                return Err(EditorError::new(
                    EditorErrorKind::InvalidInput,
                    "rename currently refuses multi-file symbols",
                ));
            }
            edits.push(LspTextEdit {
                range: location_to_range(&fol_diagnostics::DiagnosticLocation {
                    file: Some(declaration_file.clone()),
                    line: declaration.line,
                    column: declaration.column,
                    length: Some(declaration.length),
                }),
                new_text: new_name.to_string(),
            });

            for hit in program.all_references().filter(|hit| hit.resolved == Some(symbol_id)) {
                let Some(syntax_id) = hit.syntax_id else { continue };
                let Some(origin) = program.syntax_index().origin(syntax_id) else {
                    continue;
                };
                let Some(file) = origin.file.as_ref() else { continue };
                if file != &analyzed_path.to_string_lossy() {
                    return Err(EditorError::new(
                        EditorErrorKind::InvalidInput,
                        "rename currently refuses multi-file symbols",
                    ));
                }
                edits.push(LspTextEdit {
                    range: location_to_range(&fol_diagnostics::DiagnosticLocation {
                        file: Some(file.clone()),
                        line: origin.line,
                        column: origin.column,
                        length: Some(origin.length),
                    }),
                    new_text: new_name.to_string(),
                });
            }

            edits.sort_by(|left, right| {
                left.range
                    .start
                    .line
                    .cmp(&right.range.start.line)
                    .then(left.range.start.character.cmp(&right.range.start.character))
            });
            edits.dedup_by(|left, right| left == right);
            let mut changes = std::collections::BTreeMap::new();
            changes.insert(analyzed_uri, edits);
            return Ok(LspWorkspaceEdit { changes });
        }

        Err(EditorError::new(
            EditorErrorKind::InvalidInput,
            "rename target symbol was not found in the resolved workspace",
        ))
    }

    // COMPILER-BACKED: resolved symbols by path (no text fallback)
    pub(super) fn document_symbols_for_current_path(&self) -> Vec<LspDocumentSymbol> {
        let resolved = match &self.resolved_workspace {
            Some(resolved) => resolved,
            None => return Vec::new(),
        };
        let Some(analyzed_path) = &self.analyzed_path else {
            return Vec::new();
        };
        let path_text = analyzed_path.to_string_lossy();
        let mut symbols = Vec::new();
        for package in resolved.packages() {
            let program = &package.program;
            for symbol in program.all_symbols() {
                let Some(origin) = &symbol.origin else {
                    continue;
                };
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

    fn current_source_unit<'a>(
        &self,
        program: &'a fol_resolver::ResolvedProgram,
    ) -> Option<&'a fol_resolver::ResolvedSourceUnit> {
        let analyzed_path = self.analyzed_path.as_ref()?;
        let path_text = analyzed_path.to_string_lossy();
        program
            .source_units
            .iter()
            .find(move |unit| unit.path == path_text)
    }
}

#[derive(Debug, Clone)]
struct SignatureCallSite {
    callee_syntax_id: SyntaxNodeId,
    display_name: String,
    active_parameter: usize,
    span_len: usize,
}

fn visit_call_sites(
    node: &AstNode,
    program: &fol_resolver::ResolvedProgram,
    path: &std::path::Path,
    text: &str,
    cursor_offset: usize,
    best: &mut Option<SignatureCallSite>,
) {
    match node {
        AstNode::FunctionCall {
            syntax_id: Some(syntax_id),
            name,
            ..
        } => {
            if let Some(candidate) =
                signature_call_site(program, path, text, cursor_offset, *syntax_id, name)
            {
                choose_better_call_site(best, candidate);
            }
        }
        AstNode::QualifiedFunctionCall { path: qualified, .. } => {
            if let Some(syntax_id) = qualified.syntax_id() {
                if let Some(candidate) = signature_call_site(
                    program,
                    path,
                    text,
                    cursor_offset,
                    syntax_id,
                    &qualified.joined(),
                ) {
                    choose_better_call_site(best, candidate);
                }
            }
        }
        _ => {}
    }
    for child in node.children() {
        visit_call_sites(child, program, path, text, cursor_offset, best);
    }
}

fn choose_better_call_site(best: &mut Option<SignatureCallSite>, candidate: SignatureCallSite) {
    match best {
        Some(current) if current.span_len <= candidate.span_len => {}
        _ => *best = Some(candidate),
    }
}

fn signature_call_site(
    program: &fol_resolver::ResolvedProgram,
    path: &std::path::Path,
    text: &str,
    cursor_offset: usize,
    callee_syntax_id: SyntaxNodeId,
    display_name: &str,
) -> Option<SignatureCallSite> {
    let origin = program.syntax_index().origin(callee_syntax_id)?;
    if origin.file.as_deref()? != path.to_str()? {
        return None;
    }
    let callee_start = offset_for_origin(text, origin)?;
    let callee_end = callee_start + origin.length;
    let open_paren = find_call_open_paren(text, callee_end)?;
    let close_paren = find_matching_paren(text, open_paren)?;
    if cursor_offset < callee_start || cursor_offset > close_paren + 1 {
        return None;
    }
    Some(SignatureCallSite {
        callee_syntax_id,
        display_name: display_name.to_string(),
        active_parameter: active_parameter_index(text, open_paren, cursor_offset),
        span_len: close_paren.saturating_sub(callee_start),
    })
}

fn offset_for_origin(text: &str, origin: &fol_parser::ast::SyntaxOrigin) -> Option<usize> {
    offset_for_position(
        text,
        LspPosition {
            line: origin.line.saturating_sub(1) as u32,
            character: origin.column.saturating_sub(1) as u32,
        },
    )
}

fn offset_for_position(text: &str, position: LspPosition) -> Option<usize> {
    let mut offset = 0usize;
    for (line_index, line) in text.split_inclusive('\n').enumerate() {
        if line_index as u32 == position.line {
            let line_text = line.strip_suffix('\n').unwrap_or(line);
            return if position.character as usize <= line_text.len() {
                Some(offset + position.character as usize)
            } else if position.character as usize == line.len() {
                Some(offset + line.len())
            } else {
                None
            };
        }
        offset += line.len();
    }
    if position.line == text.lines().count() as u32 {
        Some(text.len())
    } else {
        None
    }
}

fn find_call_open_paren(text: &str, callee_end: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut cursor = callee_end;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'(' => return Some(cursor),
            b' ' | b'\t' | b'\r' | b'\n' => cursor += 1,
            _ => return None,
        }
    }
    None
}

fn find_matching_paren(text: &str, open_paren: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut index = open_paren;
    while index < bytes.len() {
        match bytes[index] {
            b'(' => depth += 1,
            b')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            b'"' => {
                index += 1;
                while index < bytes.len() {
                    match bytes[index] {
                        b'\\' => index += 2,
                        b'"' => break,
                        _ => index += 1,
                    }
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn active_parameter_index(text: &str, open_paren: usize, cursor_offset: usize) -> usize {
    if cursor_offset <= open_paren + 1 {
        return 0;
    }
    let bytes = text.as_bytes();
    let limit = cursor_offset.min(bytes.len());
    let mut depth = 0usize;
    let mut index = open_paren + 1;
    let mut parameter = 0usize;
    while index < limit {
        match bytes[index] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                depth = depth.saturating_sub(1);
            }
            b',' if depth == 0 => parameter += 1,
            b'"' => {
                index += 1;
                while index < limit {
                    match bytes[index] {
                        b'\\' => index += 2,
                        b'"' => break,
                        _ => index += 1,
                    }
                }
            }
            _ => {}
        }
        index += 1;
    }
    parameter
}

fn render_signature_label(
    name: &str,
    parameters: &[String],
    return_type: Option<String>,
    error_type: Option<String>,
) -> String {
    let params = parameters.join(", ");
    match (return_type, error_type) {
        (Some(returns), Some(errors)) => format!("{name}({params}): {returns} / {errors}"),
        (Some(returns), None) => format!("{name}({params}): {returns}"),
        (None, Some(errors)) => format!("{name}({params}) / {errors}"),
        (None, None) => format!("{name}({params})"),
    }
}

fn semantic_token_type_for_symbol_kind(kind: fol_resolver::SymbolKind) -> Option<u32> {
    match kind {
        fol_resolver::SymbolKind::ImportAlias => Some(0),
        fol_resolver::SymbolKind::Type | fol_resolver::SymbolKind::Alias => Some(1),
        fol_resolver::SymbolKind::Routine => Some(2),
        fol_resolver::SymbolKind::Parameter | fol_resolver::SymbolKind::GenericParameter => Some(3),
        fol_resolver::SymbolKind::ValueBinding
        | fol_resolver::SymbolKind::LabelBinding
        | fol_resolver::SymbolKind::DestructureBinding
        | fol_resolver::SymbolKind::Capture
        | fol_resolver::SymbolKind::LoopBinder
        | fol_resolver::SymbolKind::RollingBinder
        | fol_resolver::SymbolKind::Definition => Some(4),
        fol_resolver::SymbolKind::Segment
        | fol_resolver::SymbolKind::Implementation
        | fol_resolver::SymbolKind::Standard => None,
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
