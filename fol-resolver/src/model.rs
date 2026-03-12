use crate::ids::{IdTable, ImportId, ReferenceId, ScopeId, SourceUnitId, SymbolId};
use fol_parser::ast::{ParsedPackage, SyntaxIndex, SyntaxNodeId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSourceUnit {
    pub id: SourceUnitId,
    pub path: String,
    pub package: String,
    pub namespace: String,
    pub top_level_nodes: Vec<SyntaxNodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedScope {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    pub source_unit: Option<SourceUnitId>,
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
    pub source_units: IdTable<SourceUnitId, ResolvedSourceUnit>,
    pub scopes: IdTable<ScopeId, ResolvedScope>,
    pub symbols: IdTable<SymbolId, ResolvedSymbol>,
    pub references: IdTable<ReferenceId, ResolvedReference>,
    pub imports: IdTable<ImportId, ResolvedImport>,
}

impl ResolvedProgram {
    pub fn new(syntax: ParsedPackage) -> Self {
        let mut source_units = IdTable::new();

        for source_unit in &syntax.source_units {
            let top_level_nodes = source_unit.items.iter().map(|item| item.node_id).collect();
            let id = source_units.push(ResolvedSourceUnit {
                id: SourceUnitId(0),
                path: source_unit.path.clone(),
                package: source_unit.package.clone(),
                namespace: source_unit.namespace.clone(),
                top_level_nodes,
            });

            if let Some(slot) = source_units.get_mut(id) {
                slot.id = id;
            }
        }

        Self {
            syntax,
            source_units,
            scopes: IdTable::new(),
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
}
