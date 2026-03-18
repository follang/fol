use crate::{
    CheckedType, CheckedTypeId, DeclaredTypeKind, RoutineType, TypeTable, TypecheckError,
    TypecheckErrorKind, TypecheckResult, TypedProgram,
};
use fol_parser::ast::{
    AstNode, BindingPattern, FolType, Parameter, ParsedSourceUnitKind, ParsedTopLevel,
    SyntaxNodeId, SyntaxOrigin, TypeDefinition, TypeOption, VarOption,
};
use fol_resolver::{ResolvedProgram, ScopeId, SourceUnitId, SymbolId, SymbolKind};
use std::collections::BTreeMap;

pub fn lower_declaration_signatures(typed: &mut TypedProgram) -> TypecheckResult<()> {
    let resolved = typed.resolved().clone();
    let syntax = resolved.syntax().clone();
    let mut errors = Vec::new();

    for (source_unit_index, source_unit) in syntax.source_units.iter().enumerate() {
        if source_unit.kind == ParsedSourceUnitKind::Build {
            continue;
        }
        let source_unit_id = SourceUnitId(source_unit_index);
        for item in &source_unit.items {
            if let Err(error) = lower_top_level_declaration(typed, &resolved, source_unit_id, item)
            {
                errors.push(error);
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn lower_top_level_declaration(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    source_unit_id: SourceUnitId,
    item: &ParsedTopLevel,
) -> Result<(), TypecheckError> {
    if let Some(error) = unsupported_v1_top_level_decl(resolved, item) {
        return Err(error);
    }

    match &item.node {
        AstNode::VarDecl {
            name, type_hint, ..
        }
        | AstNode::LabDecl {
            name, type_hint, ..
        } => {
            if let Some(type_hint) = type_hint {
                let symbol_id = find_symbol_id(
                    resolved,
                    source_unit_id,
                    &[symbol_kind_for_node(&item.node)],
                    name,
                )?;
                let symbol_scope = resolved
                    .symbol(symbol_id)
                    .map(|symbol| symbol.scope)
                    .ok_or_else(|| internal_error("resolved binding symbol disappeared", None))?;
                let type_id = lower_type(typed, resolved, symbol_scope, type_hint)?;
                record_symbol_type(typed, symbol_id, type_id)?;
            }
        }
        AstNode::DestructureDecl {
            pattern, type_hint, ..
        } => {
            if let Some(type_hint) = type_hint {
                let binding_names = binding_names(pattern);
                let symbol_scope = binding_names
                    .first()
                    .and_then(|name| {
                        find_symbol_id(
                            resolved,
                            source_unit_id,
                            &[SymbolKind::DestructureBinding],
                            name,
                        )
                        .ok()
                    })
                    .and_then(|symbol_id| resolved.symbol(symbol_id).map(|symbol| symbol.scope))
                    .ok_or_else(|| {
                        internal_error("resolved destructure binding symbol disappeared", None)
                    })?;
                let type_id = lower_type(typed, resolved, symbol_scope, type_hint)?;
                for name in binding_names {
                    let symbol_id = find_symbol_id(
                        resolved,
                        source_unit_id,
                        &[SymbolKind::DestructureBinding],
                        &name,
                    )?;
                    record_symbol_type(typed, symbol_id, type_id)?;
                }
            }
        }
        AstNode::FunDecl {
            syntax_id,
            name,
            receiver_type,
            params,
            return_type,
            error_type,
            body,
            inquiries,
            ..
        }
        | AstNode::ProDecl {
            syntax_id,
            name,
            receiver_type,
            params,
            return_type,
            error_type,
            body,
            inquiries,
            ..
        }
        | AstNode::LogDecl {
            syntax_id,
            name,
            receiver_type,
            params,
            return_type,
            error_type,
            body,
            inquiries,
            ..
        } => {
            let signature_scope = lower_named_routine_signature(
                typed,
                resolved,
                source_unit_id,
                name,
                *syntax_id,
                receiver_type.as_ref(),
                params,
                return_type.as_ref(),
                error_type.as_ref(),
            )?;
            lower_nested_declarations_in_nodes(
                typed,
                resolved,
                source_unit_id,
                signature_scope,
                body,
            )?;
            lower_nested_declarations_in_nodes(
                typed,
                resolved,
                source_unit_id,
                signature_scope,
                inquiries,
            )?;
        }
        AstNode::TypeDecl { name, type_def, .. } => {
            let symbol_id = find_symbol_id(resolved, source_unit_id, &[SymbolKind::Type], name)?;
            let symbol_scope = resolved
                .symbol(symbol_id)
                .map(|symbol| symbol.scope)
                .ok_or_else(|| internal_error("resolved type symbol disappeared", None))?;
            let type_id = match type_def {
                TypeDefinition::Alias { target } => {
                    lower_type(typed, resolved, symbol_scope, target)?
                }
                TypeDefinition::Record { fields, .. } => {
                    let mut lowered = BTreeMap::new();
                    for (field_name, field_type) in fields {
                        lowered.insert(
                            field_name.clone(),
                            lower_type(typed, resolved, symbol_scope, field_type)?,
                        );
                    }
                    typed
                        .type_table_mut()
                        .intern(CheckedType::Record { fields: lowered })
                }
                TypeDefinition::Entry { variants, .. } => {
                    let mut lowered = BTreeMap::new();
                    for (variant_name, variant_type) in variants {
                        lowered.insert(
                            variant_name.clone(),
                            variant_type
                                .as_ref()
                                .map(|variant| lower_type(typed, resolved, symbol_scope, variant))
                                .transpose()?,
                        );
                    }
                    typed
                        .type_table_mut()
                        .intern(CheckedType::Entry { variants: lowered })
                }
            };
            record_symbol_type(typed, symbol_id, type_id)?;
        }
        AstNode::AliasDecl { name, target } => {
            let symbol_id = find_symbol_id(resolved, source_unit_id, &[SymbolKind::Alias], name)?;
            let symbol_scope = resolved
                .symbol(symbol_id)
                .map(|symbol| symbol.scope)
                .ok_or_else(|| internal_error("resolved alias symbol disappeared", None))?;
            let target_type = lower_type(typed, resolved, symbol_scope, target)?;
            record_symbol_type(typed, symbol_id, target_type)?;
        }
        _ => {}
    }

    Ok(())
}

fn lower_nested_declarations_in_nodes(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    source_unit_id: SourceUnitId,
    current_scope: ScopeId,
    nodes: &[AstNode],
) -> Result<(), TypecheckError> {
    for node in nodes {
        lower_nested_declarations_in_node(typed, resolved, source_unit_id, current_scope, node)?;
    }
    Ok(())
}

fn lower_nested_declarations_in_node(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    source_unit_id: SourceUnitId,
    current_scope: ScopeId,
    node: &AstNode,
) -> Result<(), TypecheckError> {
    if let Some(error) = unsupported_v1_nested_decl(resolved, node) {
        return Err(error);
    }

    match node {
        AstNode::VarDecl {
            name, type_hint, ..
        }
        | AstNode::LabDecl {
            name, type_hint, ..
        } => {
            if let Some(type_hint) = type_hint {
                let symbol_id = find_symbol_id_in_scope(
                    resolved,
                    source_unit_id,
                    current_scope,
                    &[symbol_kind_for_node(node)],
                    name,
                )?;
                let type_id = lower_type(typed, resolved, current_scope, type_hint)?;
                record_symbol_type(typed, symbol_id, type_id)?;
            }
        }
        AstNode::DestructureDecl {
            pattern, type_hint, ..
        } => {
            if let Some(type_hint) = type_hint {
                let type_id = lower_type(typed, resolved, current_scope, type_hint)?;
                for name in binding_names(pattern) {
                    let symbol_id = find_symbol_id_in_scope(
                        resolved,
                        source_unit_id,
                        current_scope,
                        &[SymbolKind::DestructureBinding],
                        &name,
                    )?;
                    record_symbol_type(typed, symbol_id, type_id)?;
                }
            }
        }
        AstNode::FunDecl {
            syntax_id,
            name,
            receiver_type,
            params,
            return_type,
            error_type,
            body,
            inquiries,
            ..
        }
        | AstNode::ProDecl {
            syntax_id,
            name,
            receiver_type,
            params,
            return_type,
            error_type,
            body,
            inquiries,
            ..
        }
        | AstNode::LogDecl {
            syntax_id,
            name,
            receiver_type,
            params,
            return_type,
            error_type,
            body,
            inquiries,
            ..
        } => {
            let routine_scope = lower_named_routine_signature(
                typed,
                resolved,
                source_unit_id,
                name,
                *syntax_id,
                receiver_type.as_ref(),
                params,
                return_type.as_ref(),
                error_type.as_ref(),
            )?;
            lower_nested_declarations_in_nodes(
                typed,
                resolved,
                source_unit_id,
                routine_scope,
                body,
            )?;
            lower_nested_declarations_in_nodes(
                typed,
                resolved,
                source_unit_id,
                routine_scope,
                inquiries,
            )?;
        }
        AstNode::AnonymousFun {
            params,
            return_type,
            error_type,
            ..
        }
        | AstNode::AnonymousPro {
            params,
            return_type,
            error_type,
            ..
        }
        | AstNode::AnonymousLog {
            params,
            return_type,
            error_type,
            ..
        } => {
            for param in params {
                let _ = lower_type(typed, resolved, current_scope, &param.param_type)?;
            }
            if let Some(return_type) = return_type {
                let _ = lower_type(typed, resolved, current_scope, return_type)?;
            }
            if let Some(error_type) = error_type {
                let _ = lower_type(typed, resolved, current_scope, error_type)?;
            }
        }
        AstNode::Block { statements } => {
            lower_nested_declarations_in_nodes(
                typed,
                resolved,
                source_unit_id,
                current_scope,
                statements,
            )?;
        }
        AstNode::Inquiry { body, .. } => {
            lower_nested_declarations_in_nodes(
                typed,
                resolved,
                source_unit_id,
                current_scope,
                body,
            )?;
        }
        AstNode::Commented { node, .. } => {
            lower_nested_declarations_in_node(
                typed,
                resolved,
                source_unit_id,
                current_scope,
                node,
            )?;
        }
        _ => {
            for child in node.children() {
                lower_nested_declarations_in_node(
                    typed,
                    resolved,
                    source_unit_id,
                    current_scope,
                    child,
                )?;
            }
        }
    }

    Ok(())
}

fn lower_named_routine_signature(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    source_unit_id: SourceUnitId,
    name: &str,
    syntax_id: Option<SyntaxNodeId>,
    receiver_type: Option<&FolType>,
    params: &[fol_parser::ast::Parameter],
    return_type: Option<&FolType>,
    error_type: Option<&FolType>,
) -> Result<ScopeId, TypecheckError> {
    let symbol_id = find_symbol_id(resolved, source_unit_id, &[SymbolKind::Routine], name)?;
    let signature_scope = syntax_id
        .and_then(|id| resolved.scope_for_syntax(id))
        .or_else(|| resolved.symbol(symbol_id).map(|symbol| symbol.scope))
        .ok_or_else(|| internal_error("resolved routine scope disappeared", None))?;
    let mut lowered_params = Vec::new();
    for param in params {
        let param_type = lower_type(typed, resolved, signature_scope, &param.param_type)?;
        let param_symbol_id = find_symbol_id_in_scope(
            resolved,
            source_unit_id,
            signature_scope,
            &[SymbolKind::Parameter],
            &param.name,
        )?;
        record_symbol_type(typed, param_symbol_id, param_type)?;
        lowered_params.push(param_type);
    }
    let lowered_return = match return_type {
        None | Some(FolType::None) => None,
        Some(ty) => Some(lower_type(typed, resolved, signature_scope, ty)?),
    };
    let lowered_error = error_type
        .as_ref()
        .map(|ty| lower_type(typed, resolved, signature_scope, ty))
        .transpose()?;
    let lowered_receiver = receiver_type
        .as_ref()
        .map(|ty| lower_type(typed, resolved, signature_scope, ty))
        .transpose()?;
    let routine_type = typed
        .type_table_mut()
        .intern(CheckedType::Routine(RoutineType {
            params: lowered_params,
            return_type: lowered_return,
            error_type: lowered_error,
        }));
    record_symbol_type(typed, symbol_id, routine_type)?;
    record_symbol_receiver_type(typed, symbol_id, lowered_receiver)?;
    Ok(signature_scope)
}

pub(crate) fn lower_type(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    scope_id: ScopeId,
    typ: &FolType,
) -> Result<CheckedTypeId, TypecheckError> {
    match typ {
        FolType::Int { .. } => Ok(typed.builtin_types().int),
        FolType::Float { .. } => Ok(typed.builtin_types().float),
        FolType::Bool => Ok(typed.builtin_types().bool_),
        FolType::Char { .. } => Ok(typed.builtin_types().char_),
        typ if typ.is_builtin_str() => Ok(typed.builtin_types().str_),
        FolType::Never => Ok(typed.builtin_types().never),
        FolType::Named { name, syntax_id } => {
            let symbol_id = resolved_symbol_for_syntax(
                resolved,
                *syntax_id,
                name,
                SymbolReferenceShape::Named,
            )?;
            lower_declared_symbol(typed.type_table_mut(), resolved, symbol_id)
        }
        FolType::QualifiedNamed { path } => {
            let symbol_id = resolved_symbol_for_syntax(
                resolved,
                path.syntax_id(),
                &path.joined(),
                SymbolReferenceShape::Qualified,
            )?;
            lower_declared_symbol(typed.type_table_mut(), resolved, symbol_id)
        }
        FolType::Array { element_type, size } => {
            let element_type = lower_type(typed, resolved, scope_id, element_type)?;
            Ok(typed.type_table_mut().intern(CheckedType::Array {
                element_type,
                size: *size,
            }))
        }
        FolType::Vector { element_type } => {
            let element_type = lower_type(typed, resolved, scope_id, element_type)?;
            Ok(typed
                .type_table_mut()
                .intern(CheckedType::Vector { element_type }))
        }
        FolType::Sequence { element_type } => {
            let element_type = lower_type(typed, resolved, scope_id, element_type)?;
            Ok(typed
                .type_table_mut()
                .intern(CheckedType::Sequence { element_type }))
        }
        FolType::Set { types } => {
            let mut member_types = Vec::new();
            for member in types {
                member_types.push(lower_type(typed, resolved, scope_id, member)?);
            }
            Ok(typed
                .type_table_mut()
                .intern(CheckedType::Set { member_types }))
        }
        FolType::Map {
            key_type,
            value_type,
        } => {
            let key_type = lower_type(typed, resolved, scope_id, key_type)?;
            let value_type = lower_type(typed, resolved, scope_id, value_type)?;
            Ok(typed.type_table_mut().intern(CheckedType::Map {
                key_type,
                value_type,
            }))
        }
        FolType::Optional { inner } => {
            let inner = lower_type(typed, resolved, scope_id, inner)?;
            Ok(typed
                .type_table_mut()
                .intern(CheckedType::Optional { inner }))
        }
        FolType::Error { inner } => {
            let inner = inner
                .as_ref()
                .map(|inner| lower_type(typed, resolved, scope_id, inner))
                .transpose()?;
            Ok(typed.type_table_mut().intern(CheckedType::Error { inner }))
        }
        FolType::Record { fields } => {
            let mut lowered = BTreeMap::new();
            for (field_name, field_type) in fields {
                lowered.insert(
                    field_name.clone(),
                    lower_type(typed, resolved, scope_id, field_type)?,
                );
            }
            Ok(typed
                .type_table_mut()
                .intern(CheckedType::Record { fields: lowered }))
        }
        FolType::Entry { variants } => {
            let mut lowered = BTreeMap::new();
            for (variant_name, variant_type) in variants {
                lowered.insert(
                    variant_name.clone(),
                    variant_type
                        .as_ref()
                        .map(|variant| lower_type(typed, resolved, scope_id, variant))
                        .transpose()?,
                );
            }
            Ok(typed
                .type_table_mut()
                .intern(CheckedType::Entry { variants: lowered }))
        }
        unsupported => Err(unsupported_type_error(resolved, unsupported)),
    }
}

fn lower_declared_symbol(
    table: &mut TypeTable,
    resolved: &ResolvedProgram,
    symbol_id: SymbolId,
) -> Result<CheckedTypeId, TypecheckError> {
    let symbol = resolved
        .symbol(symbol_id)
        .ok_or_else(|| internal_error("resolved type symbol disappeared", None))?;
    let kind = match symbol.kind {
        SymbolKind::Type => DeclaredTypeKind::Type,
        SymbolKind::Alias => DeclaredTypeKind::Alias,
        SymbolKind::GenericParameter => DeclaredTypeKind::GenericParameter,
        _ => {
            return Err(internal_error(
                "type reference resolved to a non-type symbol",
                symbol.origin.clone(),
            ));
        }
    };

    Ok(table.intern(CheckedType::Declared {
        symbol: symbol_id,
        name: symbol.name.clone(),
        kind,
    }))
}

fn resolved_symbol_for_syntax(
    resolved: &ResolvedProgram,
    syntax_id: Option<SyntaxNodeId>,
    display_name: &str,
    shape: SymbolReferenceShape,
) -> Result<SymbolId, TypecheckError> {
    let syntax_id = syntax_id.ok_or_else(|| {
        invalid_input_error(
            format!("type reference '{display_name}' does not retain a syntax id"),
            None,
        )
    })?;

    resolved
        .references
        .iter()
        .find(|reference| {
            reference.syntax_id == Some(syntax_id)
                && match shape {
                    SymbolReferenceShape::Named => {
                        reference.kind == fol_resolver::ReferenceKind::TypeName
                    }
                    SymbolReferenceShape::Qualified => {
                        reference.kind == fol_resolver::ReferenceKind::QualifiedTypeName
                    }
                }
        })
        .and_then(|reference| reference.resolved)
        .ok_or_else(|| {
            invalid_input_error(
                format!("type reference '{display_name}' does not have a resolved symbol"),
                resolved.syntax_index().origin(syntax_id).cloned(),
            )
        })
}

fn find_symbol_id(
    resolved: &ResolvedProgram,
    source_unit_id: SourceUnitId,
    allowed_kinds: &[SymbolKind],
    name: &str,
) -> Result<SymbolId, TypecheckError> {
    resolved
        .symbols
        .iter_with_ids()
        .find(|(_, symbol)| {
            symbol.source_unit == source_unit_id
                && symbol.name == name
                && allowed_kinds.contains(&symbol.kind)
        })
        .map(|(symbol_id, _)| symbol_id)
        .ok_or_else(|| {
            internal_error(
                format!("resolved declaration symbol '{name}' is missing from typed lowering"),
                None,
            )
        })
}

fn find_symbol_id_in_scope(
    resolved: &ResolvedProgram,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    allowed_kinds: &[SymbolKind],
    name: &str,
) -> Result<SymbolId, TypecheckError> {
    resolved
        .symbols
        .iter_with_ids()
        .find(|(_, symbol)| {
            symbol.source_unit == source_unit_id
                && symbol.scope == scope_id
                && symbol.name == name
                && allowed_kinds.contains(&symbol.kind)
        })
        .map(|(symbol_id, _)| symbol_id)
        .ok_or_else(|| {
            internal_error(
                format!(
                    "resolved declaration symbol '{name}' is missing from typed lowering for scope {}",
                    scope_id.0
                ),
                None,
            )
        })
}

fn record_symbol_type(
    typed: &mut TypedProgram,
    symbol_id: SymbolId,
    type_id: CheckedTypeId,
) -> Result<(), TypecheckError> {
    let symbol = typed.typed_symbol_mut(symbol_id).ok_or_else(|| {
        TypecheckError::new(
            TypecheckErrorKind::SymbolTableCorrupted,
            format!(
                "symbol table corrupted: symbol {} is missing while recording declared type {}",
                symbol_id.0, type_id.0,
            ),
        )
    })?;
    symbol.declared_type = Some(type_id);
    Ok(())
}

fn record_symbol_receiver_type(
    typed: &mut TypedProgram,
    symbol_id: SymbolId,
    type_id: Option<CheckedTypeId>,
) -> Result<(), TypecheckError> {
    let symbol = typed.typed_symbol_mut(symbol_id).ok_or_else(|| {
        TypecheckError::new(
            TypecheckErrorKind::SymbolTableCorrupted,
            format!(
                "symbol table corrupted: symbol {} is missing while recording receiver type",
                symbol_id.0,
            ),
        )
    })?;
    symbol.receiver_type = type_id;
    Ok(())
}

fn binding_names(pattern: &BindingPattern) -> Vec<String> {
    match pattern {
        BindingPattern::Name(name) | BindingPattern::Rest(name) => vec![name.clone()],
        BindingPattern::Sequence(parts) => parts.iter().flat_map(binding_names).collect(),
    }
}

fn symbol_kind_for_node(node: &AstNode) -> SymbolKind {
    match node {
        AstNode::VarDecl { .. } => SymbolKind::ValueBinding,
        AstNode::LabDecl { .. } => SymbolKind::LabelBinding,
        _ => SymbolKind::ValueBinding,
    }
}

fn unsupported_type_error(resolved: &ResolvedProgram, typ: &FolType) -> TypecheckError {
    let label = match typ {
        FolType::Matrix { .. } => "matrix types are not part of the V1 typecheck milestone",
        FolType::Pointer { .. } => {
            "pointer types are part of the V3 systems milestone, not the V1 typecheck milestone"
        }
        FolType::Channel { .. } => "channel types are not part of the V1 typecheck milestone",
        FolType::Multiple { .. } => "multiple types are not part of the V1 typecheck milestone",
        FolType::Union { .. } => "union types are not part of the V1 typecheck milestone",
        FolType::Limited { .. } => "limited types are not part of the V1 typecheck milestone",
        FolType::Any => "any types are not part of the V1 typecheck milestone",
        FolType::None => "none types are not part of the V1 typecheck milestone",
        FolType::Function { .. } => {
            "function type literals are not part of the V1 typecheck milestone"
        }
        FolType::Generic { .. } => {
            "generic type parameters are not part of the V1 typecheck milestone"
        }
        FolType::Package { .. }
        | FolType::Module { .. }
        | FolType::Block { .. }
        | FolType::Test { .. }
        | FolType::Location { .. }
        | FolType::Standard { .. } => {
            "package/build-specific type surfaces are not part of the V1 typecheck milestone"
        }
        _ => "type surface is not part of the V1 typecheck milestone",
    };
    match type_origin(resolved, typ) {
        Some(origin) => TypecheckError::with_origin(TypecheckErrorKind::Unsupported, label, origin),
        None => TypecheckError::new(TypecheckErrorKind::Unsupported, label),
    }
}

fn unsupported_v1_top_level_decl(
    resolved: &ResolvedProgram,
    item: &ParsedTopLevel,
) -> Option<TypecheckError> {
    let origin = resolved.syntax_index().origin(item.node_id).cloned();
    unsupported_v1_decl_with_origin(&item.node, origin)
}

fn unsupported_v1_nested_decl(
    resolved: &ResolvedProgram,
    node: &AstNode,
) -> Option<TypecheckError> {
    unsupported_v1_decl_with_origin(node, node_origin(resolved, node))
}

fn unsupported_v1_decl_with_origin(
    node: &AstNode,
    origin: Option<SyntaxOrigin>,
) -> Option<TypecheckError> {
    let message = match node {
        AstNode::VarDecl { options, .. } | AstNode::LabDecl { options, .. } => {
            unsupported_binding_surface_message(options)
        }
        AstNode::FunDecl { generics, .. }
        | AstNode::ProDecl { generics, .. }
        | AstNode::LogDecl { generics, .. }
            if !generics.is_empty() =>
        {
            Some("generic routine semantics are not part of the V1 typecheck milestone")
        }
        AstNode::FunDecl { params, .. }
        | AstNode::ProDecl { params, .. }
        | AstNode::LogDecl { params, .. } => unsupported_routine_param_surface_message(params),
        AstNode::TypeDecl { contracts, .. } if !contracts.is_empty() => {
            Some("type contract conformance is part of the V2 language milestone, not the V1 typecheck milestone")
        }
        AstNode::TypeDecl { options, .. }
            if options
                .iter()
                .any(|option| matches!(option, TypeOption::Extension)) =>
        {
            Some("type extension declarations are part of the V2 language milestone, not the V1 typecheck milestone")
        }
        AstNode::TypeDecl { generics, .. } if !generics.is_empty() => {
            Some("generic type semantics are not part of the V1 typecheck milestone")
        }
        AstNode::DefDecl { .. } => {
            Some("definition/meta declarations are part of the V2 language milestone, not the V1 typecheck milestone")
        }
        AstNode::SegDecl { .. } => {
            Some("segment/meta declarations are part of the V2 language milestone, not the V1 typecheck milestone")
        }
        AstNode::ImpDecl { .. } => {
            Some("implementation declarations are part of the V2 language milestone, not the V1 typecheck milestone")
        }
        AstNode::StdDecl { kind, .. } => Some(match kind {
            fol_parser::ast::StandardKind::Protocol => {
                "protocol standards are part of the V2 language milestone, not the V1 typecheck milestone"
            }
            fol_parser::ast::StandardKind::Blueprint => {
                "blueprint standards are part of the V2 language milestone, not the V1 typecheck milestone"
            }
            fol_parser::ast::StandardKind::Extended => {
                "extended standards are part of the V2 language milestone, not the V1 typecheck milestone"
            }
        }),
        _ => None,
    }?;

    Some(match origin {
        Some(origin) => {
            TypecheckError::with_origin(TypecheckErrorKind::Unsupported, message, origin)
        }
        None => TypecheckError::new(TypecheckErrorKind::Unsupported, message),
    })
}

pub(crate) fn unsupported_routine_param_surface_message(
    params: &[Parameter],
) -> Option<&'static str> {
    if params.iter().any(|param| param.is_mutex) {
        Some("mutex parameter semantics are part of the V3 systems milestone, not the V1 typecheck milestone")
    } else if params.iter().any(|param| param.is_borrowable) {
        Some("borrowable parameter semantics are part of the V3 systems milestone, not the V1 typecheck milestone")
    } else {
        None
    }
}

fn unsupported_binding_surface_message(options: &[VarOption]) -> Option<&'static str> {
    if options
        .iter()
        .any(|option| matches!(option, VarOption::Borrowing))
    {
        Some("borrowing binding semantics are part of the V3 systems milestone, not the V1 typecheck milestone")
    } else if options
        .iter()
        .any(|option| matches!(option, VarOption::New))
    {
        Some("heap/new binding semantics are part of the V3 systems milestone, not the V1 typecheck milestone")
    } else if options
        .iter()
        .any(|option| matches!(option, VarOption::Static))
    {
        Some("static binding semantics are not part of the V1 typecheck milestone")
    } else if options
        .iter()
        .any(|option| matches!(option, VarOption::Reactive))
    {
        Some("reactive binding semantics are not part of the V1 typecheck milestone")
    } else {
        None
    }
}

fn type_origin(resolved: &ResolvedProgram, typ: &FolType) -> Option<SyntaxOrigin> {
    match typ {
        FolType::Named { syntax_id, .. } => syntax_id
            .and_then(|syntax_id| resolved.syntax_index().origin(syntax_id))
            .cloned(),
        FolType::QualifiedNamed { path } => path
            .syntax_id()
            .and_then(|syntax_id| resolved.syntax_index().origin(syntax_id))
            .cloned(),
        _ => None,
    }
}

fn node_origin(resolved: &ResolvedProgram, node: &AstNode) -> Option<SyntaxOrigin> {
    if let Some(syntax_id) = node.syntax_id() {
        return resolved.syntax_index().origin(syntax_id).cloned();
    }

    for child in node.children() {
        if let Some(origin) = node_origin(resolved, child) {
            return Some(origin);
        }
    }

    None
}

fn invalid_input_error(message: impl Into<String>, origin: Option<SyntaxOrigin>) -> TypecheckError {
    match origin {
        Some(origin) => {
            TypecheckError::with_origin(TypecheckErrorKind::InvalidInput, message, origin)
        }
        None => TypecheckError::new(TypecheckErrorKind::InvalidInput, message),
    }
}

fn internal_error(message: impl Into<String>, origin: Option<SyntaxOrigin>) -> TypecheckError {
    match origin {
        Some(origin) => TypecheckError::with_origin(TypecheckErrorKind::Internal, message, origin),
        None => TypecheckError::new(TypecheckErrorKind::Internal, message),
    }
}

enum SymbolReferenceShape {
    Named,
    Qualified,
}
