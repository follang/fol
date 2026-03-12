use crate::ids::{IdTable, ImportId, ReferenceId, ScopeId, SourceUnitId, SymbolId};
use fol_parser::ast::{ParsedPackage, SyntaxIndex, SyntaxNodeId};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeKind {
    ProgramRoot { package: String },
    NamespaceRoot { namespace: String },
    SourceUnitRoot { path: String },
    Routine,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSymbol {
    pub id: SymbolId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedReference {
    pub id: ReferenceId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedImport {
    pub id: ImportId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedProgram {
    syntax: ParsedPackage,
    pub program_scope: ScopeId,
    namespace_scopes: BTreeMap<String, ScopeId>,
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
