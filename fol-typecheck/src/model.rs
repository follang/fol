use crate::{BuiltinTypeIds, CheckedTypeId, TypeTable};
use fol_intrinsics::IntrinsicId;
use fol_parser::ast::SyntaxNodeId;
use fol_resolver::{PackageIdentity, ReferenceKind, ScopeId, SourceUnitId, SymbolId, SymbolKind};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoverableCallEffect {
    pub error_type: CheckedTypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedExportMount {
    pub source_namespace: String,
    pub mounted_namespace_suffix: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedSourceUnit {
    pub source_unit_id: SourceUnitId,
    pub path: String,
    pub package: String,
    pub namespace: String,
    pub scope_id: ScopeId,
    pub top_level_nodes: Vec<SyntaxNodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedSymbol {
    pub symbol_id: SymbolId,
    pub kind: SymbolKind,
    pub scope_id: ScopeId,
    pub source_unit_id: SourceUnitId,
    pub declared_type: Option<CheckedTypeId>,
    pub receiver_type: Option<CheckedTypeId>,
    pub recoverable_effect: Option<RecoverableCallEffect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedNode {
    pub syntax_id: SyntaxNodeId,
    pub source_unit_id: SourceUnitId,
    pub inferred_type: Option<CheckedTypeId>,
    pub recoverable_effect: Option<RecoverableCallEffect>,
    pub intrinsic_id: Option<IntrinsicId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedReference {
    pub reference_id: fol_resolver::ReferenceId,
    pub kind: ReferenceKind,
    pub source_unit_id: SourceUnitId,
    pub resolved_type: Option<CheckedTypeId>,
    pub recoverable_effect: Option<RecoverableCallEffect>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedProgram {
    resolved: fol_resolver::ResolvedProgram,
    type_table: TypeTable,
    builtins: BuiltinTypeIds,
    source_units: Vec<TypedSourceUnit>,
    symbols: BTreeMap<SymbolId, TypedSymbol>,
    nodes: BTreeMap<SyntaxNodeId, TypedNode>,
    references: BTreeMap<fol_resolver::ReferenceId, TypedReference>,
    apparent_type_overrides: BTreeMap<CheckedTypeId, CheckedTypeId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedPackage {
    pub identity: PackageIdentity,
    pub export_mounts: Vec<TypedExportMount>,
    pub program: TypedProgram,
}

impl TypedPackage {
    pub fn new(
        identity: PackageIdentity,
        export_mounts: Vec<TypedExportMount>,
        program: TypedProgram,
    ) -> Self {
        Self {
            identity,
            export_mounts,
            program,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedWorkspace {
    entry_identity: PackageIdentity,
    packages: BTreeMap<PackageIdentity, TypedPackage>,
}

impl TypedWorkspace {
    pub fn single(entry_identity: PackageIdentity, entry_program: TypedProgram) -> Self {
        let mut packages = BTreeMap::new();
        packages.insert(
            entry_identity.clone(),
            TypedPackage::new(entry_identity.clone(), Vec::new(), entry_program),
        );
        Self {
            entry_identity,
            packages,
        }
    }

    pub(crate) fn new(
        entry_identity: PackageIdentity,
        packages: BTreeMap<PackageIdentity, TypedPackage>,
    ) -> Self {
        Self {
            entry_identity,
            packages,
        }
    }

    pub fn entry_identity(&self) -> &PackageIdentity {
        &self.entry_identity
    }

    pub fn entry_package(&self) -> &TypedPackage {
        self.packages
            .get(&self.entry_identity)
            .expect("typed workspace should always retain the entry package")
    }

    pub fn entry_program(&self) -> &TypedProgram {
        &self.entry_package().program
    }

    pub fn package(&self, identity: &PackageIdentity) -> Option<&TypedPackage> {
        self.packages.get(identity)
    }

    pub fn packages(&self) -> impl Iterator<Item = &TypedPackage> {
        self.packages.values()
    }

    pub fn package_count(&self) -> usize {
        self.packages.len()
    }
}

impl TypedProgram {
    pub fn from_resolved(resolved: fol_resolver::ResolvedProgram) -> Self {
        let mut type_table = TypeTable::new();
        let builtins = BuiltinTypeIds::install(&mut type_table);
        let source_units = resolved
            .source_units
            .iter_with_ids()
            .map(|(source_unit_id, unit)| TypedSourceUnit {
                source_unit_id,
                path: unit.path.clone(),
                package: unit.package.clone(),
                namespace: unit.namespace.clone(),
                scope_id: unit.scope_id,
                top_level_nodes: unit.top_level_nodes.clone(),
            })
            .collect::<Vec<_>>();
        let symbols = resolved
            .symbols
            .iter_with_ids()
            .map(|(symbol_id, symbol)| {
                (
                    symbol_id,
                    TypedSymbol {
                        symbol_id,
                        kind: symbol.kind,
                        scope_id: symbol.scope,
                        source_unit_id: symbol.source_unit,
                        declared_type: None,
                        receiver_type: None,
                        recoverable_effect: None,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        let nodes = source_units
            .iter()
            .flat_map(|unit| {
                unit.top_level_nodes.iter().copied().map(move |syntax_id| {
                    (
                        syntax_id,
                        TypedNode {
                            syntax_id,
                            source_unit_id: unit.source_unit_id,
                            inferred_type: None,
                            recoverable_effect: None,
                            intrinsic_id: None,
                        },
                    )
                })
            })
            .collect::<BTreeMap<_, _>>();
        let references = resolved
            .references
            .iter_with_ids()
            .map(|(reference_id, reference)| {
                (
                    reference_id,
                    TypedReference {
                        reference_id,
                        kind: reference.kind,
                        source_unit_id: reference.source_unit,
                        resolved_type: None,
                        recoverable_effect: None,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();

        Self {
            resolved,
            type_table,
            builtins,
            source_units,
            symbols,
            nodes,
            references,
            apparent_type_overrides: BTreeMap::new(),
        }
    }

    pub fn package_name(&self) -> &str {
        self.resolved.package_name()
    }

    pub fn resolved(&self) -> &fol_resolver::ResolvedProgram {
        &self.resolved
    }

    pub fn type_table(&self) -> &TypeTable {
        &self.type_table
    }

    pub fn builtin_types(&self) -> BuiltinTypeIds {
        self.builtins
    }

    pub(crate) fn type_table_mut(&mut self) -> &mut TypeTable {
        &mut self.type_table
    }

    pub fn source_units(&self) -> &[TypedSourceUnit] {
        &self.source_units
    }

    pub fn typed_symbol(&self, symbol_id: SymbolId) -> Option<&TypedSymbol> {
        self.symbols.get(&symbol_id)
    }

    pub(crate) fn typed_symbol_mut(&mut self, symbol_id: SymbolId) -> Option<&mut TypedSymbol> {
        self.symbols.get_mut(&symbol_id)
    }

    pub fn typed_node(&self, syntax_id: SyntaxNodeId) -> Option<&TypedNode> {
        self.nodes.get(&syntax_id)
    }

    pub fn typed_reference(
        &self,
        reference_id: fol_resolver::ReferenceId,
    ) -> Option<&TypedReference> {
        self.references.get(&reference_id)
    }

    pub(crate) fn typed_reference_mut(
        &mut self,
        reference_id: fol_resolver::ReferenceId,
    ) -> Option<&mut TypedReference> {
        self.references.get_mut(&reference_id)
    }

    pub(crate) fn record_node_type(
        &mut self,
        syntax_id: SyntaxNodeId,
        source_unit_id: SourceUnitId,
        type_id: CheckedTypeId,
    ) -> Result<(), crate::TypecheckError> {
        self.nodes
            .entry(syntax_id)
            .or_insert(TypedNode {
                syntax_id,
                source_unit_id,
                inferred_type: None,
                recoverable_effect: None,
                intrinsic_id: None,
            })
            .inferred_type = Some(type_id);
        Ok(())
    }

    pub(crate) fn record_node_recoverable_effect(
        &mut self,
        syntax_id: SyntaxNodeId,
        source_unit_id: SourceUnitId,
        effect: RecoverableCallEffect,
    ) -> Result<(), crate::TypecheckError> {
        self.nodes
            .entry(syntax_id)
            .or_insert(TypedNode {
                syntax_id,
                source_unit_id,
                inferred_type: None,
                recoverable_effect: None,
                intrinsic_id: None,
            })
            .recoverable_effect = Some(effect);
        Ok(())
    }

    pub(crate) fn record_node_intrinsic(
        &mut self,
        syntax_id: SyntaxNodeId,
        source_unit_id: SourceUnitId,
        intrinsic_id: IntrinsicId,
    ) -> Result<(), crate::TypecheckError> {
        self.nodes
            .entry(syntax_id)
            .or_insert(TypedNode {
                syntax_id,
                source_unit_id,
                inferred_type: None,
                recoverable_effect: None,
                intrinsic_id: None,
            })
            .intrinsic_id = Some(intrinsic_id);
        Ok(())
    }

    pub(crate) fn record_reference_recoverable_effect(
        &mut self,
        reference_id: fol_resolver::ReferenceId,
        effect: RecoverableCallEffect,
    ) -> Result<(), crate::TypecheckError> {
        let reference = self
            .typed_reference_mut(reference_id)
            .ok_or_else(|| crate::TypecheckError::new(crate::TypecheckErrorKind::Internal, "typed reference disappeared while recording a recoverable call effect"))?;
        reference.recoverable_effect = Some(effect);
        Ok(())
    }

    pub(crate) fn record_apparent_type_override(
        &mut self,
        shell_type: CheckedTypeId,
        apparent_type: CheckedTypeId,
    ) {
        self.apparent_type_overrides.insert(shell_type, apparent_type);
    }

    pub(crate) fn apparent_type_override(
        &self,
        type_id: CheckedTypeId,
    ) -> Option<CheckedTypeId> {
        self.apparent_type_overrides.get(&type_id).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::{TypedProgram, TypedWorkspace};
    use crate::{BuiltinType, CheckedType};
    use fol_parser::ast::AstParser;
    use fol_resolver::resolve_package;
    use fol_stream::FileStream;

    #[test]
    fn typed_program_shell_installs_builtin_types_for_resolved_programs() {
        let fixture_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/parser/simple_var.fol");
        let mut stream = FileStream::from_file(fixture_path).expect("Should open typecheck fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("Typecheck fixture should parse");
        let resolved = resolve_package(syntax).expect("Typecheck fixture should resolve");

        let typed = TypedProgram::from_resolved(resolved);

        assert_eq!(typed.package_name(), "parser");
        assert_eq!(
            typed.type_table().get(typed.builtin_types().str_),
            Some(&CheckedType::Builtin(BuiltinType::Str))
        );
        assert_eq!(typed.source_units().len(), 1);
        assert_eq!(
            typed.source_units()[0].top_level_nodes,
            typed.resolved().source_units.get(fol_resolver::SourceUnitId(0))
                .expect("resolved source unit should exist")
                .top_level_nodes
        );
    }

    #[test]
    fn typed_workspace_single_package_shell_exposes_entry_program() {
        let fixture_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/parser/simple_var.fol");
        let mut stream = FileStream::from_file(fixture_path).expect("Should open typecheck fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("Typecheck fixture should parse");
        let resolved = resolve_package(syntax).expect("Typecheck fixture should resolve");
        let entry_identity = fol_resolver::PackageIdentity {
            source_kind: fol_resolver::PackageSourceKind::Entry,
            canonical_root: resolved.package_name().to_string(),
            display_name: resolved.package_name().to_string(),
        };

        let workspace = TypedWorkspace::single(entry_identity.clone(), TypedProgram::from_resolved(resolved));

        assert_eq!(workspace.package_count(), 1);
        assert_eq!(workspace.entry_identity(), &entry_identity);
        assert_eq!(workspace.entry_program().package_name(), "parser");
    }
}
