use crate::{location_to_range, EditorDocument, LspDiagnostic, LspLocation, LspPosition, LspRange};
use fol_intrinsics::{
    intrinsic_registry, IntrinsicAvailability, IntrinsicStatus, IntrinsicSurface,
};
use std::path::PathBuf;

use super::completion_helpers::{
    completion_builtin_type_item, completion_context,
    completion_intrinsic_item, completion_item_from_symbol, completion_namespace_item,
    completion_symbol_is_plain_top_level_candidate, completion_symbol_is_root_visible,
    current_routine_name, dedupe_completion_items, fallback_decl_name,
    fallback_items_from_package_dir, position_to_offset, render_checked_type, render_symbol_kind,
    symbol_kind_code, symbol_visibility_matches_namespace_root, CompletionContext,
};
use super::types::{EditorCompletionItem, LspDocumentSymbol, LspHover};

pub(super) struct SemanticSnapshot {
    pub(super) analyzed_path: Option<PathBuf>,
    pub(super) source_document_path: PathBuf,
    pub(super) source_package_root: Option<PathBuf>,
    pub(super) diagnostics: Vec<LspDiagnostic>,
    pub(super) resolved_workspace: Option<fol_resolver::ResolvedWorkspace>,
    pub(super) typed_workspace: Option<fol_typecheck::TypedWorkspace>,
}

impl SemanticSnapshot {
    pub(super) fn plain_completion_items(
        &self,
        document: &EditorDocument,
        position: LspPosition,
    ) -> Vec<EditorCompletionItem> {
        if self.current_program().is_none() {
            return self.fallback_completion_items(document, position);
        }
        match completion_context(document, position) {
            CompletionContext::Plain => {}
            CompletionContext::TypePosition => {
                let mut items = self.builtin_type_completion_items();
                items.extend(self.visible_named_type_completion_items());
                items.extend(self.fallback_local_named_type_items(document));
                items.extend(self.fallback_imported_named_type_items(document));
                return dedupe_completion_items(items);
            }
            CompletionContext::QualifiedPath { qualifier } => {
                let mut items = self.qualified_completion_items(&qualifier);
                items.extend(self.fallback_qualified_completion_items(&qualifier));
                return dedupe_completion_items(items);
            }
            CompletionContext::DotTrigger => return self.dot_intrinsic_fallback_completion_items(),
        }
        let mut items = self.local_completion_items(position);
        items.extend(self.current_package_top_level_completion_items());
        items.extend(self.import_alias_completion_items(position));
        items.extend(self.fallback_local_scope_items(document, position));
        items.extend(self.fallback_current_package_top_level_items(document, position));
        items.extend(self.fallback_import_alias_items(document));
        dedupe_completion_items(items)
    }

    fn fallback_completion_items(
        &self,
        document: &EditorDocument,
        position: LspPosition,
    ) -> Vec<EditorCompletionItem> {
        match completion_context(document, position) {
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
