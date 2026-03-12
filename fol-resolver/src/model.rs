use crate::ids::{IdTable, ImportId, ReferenceId, ScopeId, SourceUnitId, SymbolId};
use fol_parser::ast::{
    FolType, ParsedDeclScope, ParsedDeclVisibility, ParsedPackage, SyntaxIndex, SyntaxNodeId,
    SyntaxOrigin, UsePathSegment,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeKind {
    ProgramRoot { package: String },
    NamespaceRoot { namespace: String },
    SourceUnitRoot { path: String },
    Routine,
    TypeDeclaration,
    Block,
    LoopBinder,
    RollingBinder,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSourceUnit {
    pub id: SourceUnitId,
    pub path: String,
    pub package: String,
    pub namespace: String,
    pub scope_id: ScopeId,
    pub top_level_nodes: Vec<SyntaxNodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedScope {
    pub id: ScopeId,
    pub kind: ScopeKind,
    pub parent: Option<ScopeId>,
    pub source_unit: Option<SourceUnitId>,
    pub symbols: Vec<SymbolId>,
    pub symbol_keys: BTreeMap<String, Vec<SymbolId>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    ValueBinding,
    LabelBinding,
    DestructureBinding,
    Routine,
    Type,
    Alias,
    Definition,
    Segment,
    Implementation,
    Standard,
    ImportAlias,
    GenericParameter,
    Parameter,
    Capture,
    LoopBinder,
    RollingBinder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceKind {
    Identifier,
    FunctionCall,
    QualifiedIdentifier,
    QualifiedFunctionCall,
    TypeName,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSymbol {
    pub id: SymbolId,
    pub name: String,
    pub canonical_name: String,
    pub duplicate_key: String,
    pub kind: SymbolKind,
    pub scope: ScopeId,
    pub source_unit: SourceUnitId,
    pub origin: Option<SyntaxOrigin>,
    pub visibility: Option<ParsedDeclVisibility>,
    pub declaration_scope: Option<ParsedDeclScope>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedReference {
    pub id: ReferenceId,
    pub kind: ReferenceKind,
    pub name: String,
    pub scope: ScopeId,
    pub source_unit: SourceUnitId,
    pub resolved: Option<SymbolId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedImport {
    pub id: ImportId,
    pub alias_symbol: SymbolId,
    pub alias_name: String,
    pub path_type: FolType,
    pub path_segments: Vec<UsePathSegment>,
    pub scope: ScopeId,
    pub source_unit: SourceUnitId,
    pub target_scope: Option<ScopeId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedProgram {
    syntax: ParsedPackage,
    pub program_scope: ScopeId,
    namespace_scopes: BTreeMap<String, ScopeId>,
    syntax_scopes: BTreeMap<SyntaxNodeId, ScopeId>,
    pub source_units: IdTable<SourceUnitId, ResolvedSourceUnit>,
    pub scopes: IdTable<ScopeId, ResolvedScope>,
    pub symbols: IdTable<SymbolId, ResolvedSymbol>,
    pub references: IdTable<ReferenceId, ResolvedReference>,
    pub imports: IdTable<ImportId, ResolvedImport>,
}

impl ResolvedProgram {
    pub fn new(syntax: ParsedPackage) -> Self {
        let mut scopes = IdTable::new();
        let program_scope = scopes.push(ResolvedScope {
            id: ScopeId(0),
            kind: ScopeKind::ProgramRoot {
                package: syntax.package.clone(),
            },
            parent: None,
            source_unit: None,
            symbols: Vec::new(),
            symbol_keys: BTreeMap::new(),
        });
        if let Some(scope) = scopes.get_mut(program_scope) {
            scope.id = program_scope;
        }

        let mut namespace_scopes = BTreeMap::new();
        namespace_scopes.insert(syntax.package.clone(), program_scope);
        let mut source_units = IdTable::new();

        for source_unit in &syntax.source_units {
            let top_level_nodes = source_unit.items.iter().map(|item| item.node_id).collect();
            let id = source_units.push(ResolvedSourceUnit {
                id: SourceUnitId(0),
                path: source_unit.path.clone(),
                package: source_unit.package.clone(),
                namespace: source_unit.namespace.clone(),
                scope_id: ScopeId(0),
                top_level_nodes,
            });
            if let Some(slot) = source_units.get_mut(id) {
                slot.id = id;
            }

            ensure_namespace_scope(
                &mut scopes,
                &mut namespace_scopes,
                program_scope,
                &source_unit.package,
                &source_unit.namespace,
            );
        }

        let source_unit_ids = source_units
            .iter_with_ids()
            .map(|(id, _)| id)
            .collect::<Vec<_>>();

        for source_unit_id in source_unit_ids {
            let source_unit = source_units
                .get(source_unit_id)
                .expect("source unit should exist while building source-unit scopes");
            let source_scope = scopes.push(ResolvedScope {
                id: ScopeId(0),
                kind: ScopeKind::SourceUnitRoot {
                    path: source_unit.path.clone(),
                },
                parent: Some(
                    *namespace_scopes
                        .get(&source_unit.namespace)
                        .expect("source unit namespace scope should exist"),
                ),
                source_unit: Some(source_unit_id),
                symbols: Vec::new(),
                symbol_keys: BTreeMap::new(),
            });
            if let Some(scope) = scopes.get_mut(source_scope) {
                scope.id = source_scope;
            }
            if let Some(unit) = source_units.get_mut(source_unit_id) {
                unit.scope_id = source_scope;
            }
        }

        Self {
            syntax,
            program_scope,
            namespace_scopes,
            syntax_scopes: BTreeMap::new(),
            source_units,
            scopes,
            symbols: IdTable::new(),
            references: IdTable::new(),
            imports: IdTable::new(),
        }
    }

    pub fn syntax(&self) -> &ParsedPackage {
        &self.syntax
    }

    pub fn syntax_index(&self) -> &SyntaxIndex {
        &self.syntax.syntax_index
    }

    pub fn package_name(&self) -> &str {
        &self.syntax.package
    }

    pub fn scope(&self, id: ScopeId) -> Option<&ResolvedScope> {
        self.scopes.get(id)
    }

    pub fn source_unit(&self, id: SourceUnitId) -> Option<&ResolvedSourceUnit> {
        self.source_units.get(id)
    }

    pub fn namespace_scope(&self, namespace: &str) -> Option<ScopeId> {
        self.namespace_scopes.get(namespace).copied()
    }

    pub fn scope_for_syntax(&self, syntax_id: SyntaxNodeId) -> Option<ScopeId> {
        self.syntax_scopes.get(&syntax_id).copied()
    }

    pub fn symbol(&self, id: SymbolId) -> Option<&ResolvedSymbol> {
        self.symbols.get(id)
    }

    pub fn reference(&self, id: ReferenceId) -> Option<&ResolvedReference> {
        self.references.get(id)
    }

    pub fn import(&self, id: ImportId) -> Option<&ResolvedImport> {
        self.imports.get(id)
    }

    pub fn symbols_in_scope(&self, scope_id: ScopeId) -> Vec<&ResolvedSymbol> {
        self.scope(scope_id)
            .map(|scope| {
                scope
                    .symbols
                    .iter()
                    .filter_map(|symbol_id| self.symbol(*symbol_id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn symbols_named_in_scope(&self, scope_id: ScopeId, key: &str) -> Vec<&ResolvedSymbol> {
        self.scope(scope_id)
            .and_then(|scope| scope.symbol_keys.get(key))
            .map(|ids| ids.iter().filter_map(|id| self.symbol(*id)).collect())
            .unwrap_or_default()
    }

    pub fn references_in_scope(&self, scope_id: ScopeId) -> Vec<&ResolvedReference> {
        self.references
            .iter()
            .filter(|reference| reference.scope == scope_id)
            .collect()
    }

    pub fn imports_in_scope(&self, scope_id: ScopeId) -> Vec<&ResolvedImport> {
        self.imports
            .iter()
            .filter(|import| import.scope == scope_id)
            .collect()
    }

    pub(crate) fn add_scope(
        &mut self,
        kind: ScopeKind,
        parent: ScopeId,
        source_unit: SourceUnitId,
    ) -> ScopeId {
        let scope_id = self.scopes.push(ResolvedScope {
            id: ScopeId(0),
            kind,
            parent: Some(parent),
            source_unit: Some(source_unit),
            symbols: Vec::new(),
            symbol_keys: BTreeMap::new(),
        });
        if let Some(scope) = self.scopes.get_mut(scope_id) {
            scope.id = scope_id;
        }
        scope_id
    }

    pub(crate) fn record_scope_for_syntax(
        &mut self,
        syntax_id: Option<SyntaxNodeId>,
        scope_id: ScopeId,
    ) {
        if let Some(syntax_id) = syntax_id {
            self.syntax_scopes.insert(syntax_id, scope_id);
        }
    }
}

fn ensure_namespace_scope(
    scopes: &mut IdTable<ScopeId, ResolvedScope>,
    namespace_scopes: &mut BTreeMap<String, ScopeId>,
    program_scope: ScopeId,
    package: &str,
    namespace: &str,
) -> ScopeId {
    if let Some(scope_id) = namespace_scopes.get(namespace) {
        return *scope_id;
    }

    let mut parent_scope = program_scope;
    let mut current_namespace = package.to_string();

    for segment in namespace.split("::").skip(1) {
        current_namespace.push_str("::");
        current_namespace.push_str(segment);

        if let Some(scope_id) = namespace_scopes.get(&current_namespace) {
            parent_scope = *scope_id;
            continue;
        }

        let scope_id = scopes.push(ResolvedScope {
            id: ScopeId(0),
            kind: ScopeKind::NamespaceRoot {
                namespace: current_namespace.clone(),
            },
            parent: Some(parent_scope),
            source_unit: None,
            symbols: Vec::new(),
            symbol_keys: BTreeMap::new(),
        });
        if let Some(scope) = scopes.get_mut(scope_id) {
            scope.id = scope_id;
        }
        namespace_scopes.insert(current_namespace.clone(), scope_id);
        parent_scope = scope_id;
    }

    parent_scope
}
