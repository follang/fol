use crate::{CheckedTypeId, TypecheckError, TypecheckErrorKind, TypecheckResult, TypedProgram};
use fol_parser::ast::{AstNode, Literal, QualifiedPath, SyntaxNodeId, SyntaxOrigin};
use fol_resolver::{ReferenceId, ReferenceKind, ResolvedProgram, SourceUnitId, SymbolId};

#[derive(Debug, Clone, Copy)]
struct TypeContext {
    source_unit_id: SourceUnitId,
}

pub fn type_program(typed: &mut TypedProgram) -> TypecheckResult<()> {
    let resolved = typed.resolved().clone();
    let syntax = resolved.syntax().clone();
    let mut errors = Vec::new();

    for (source_unit_index, source_unit) in syntax.source_units.iter().enumerate() {
        let context = TypeContext {
            source_unit_id: SourceUnitId(source_unit_index),
        };
        for item in &source_unit.items {
            if let Err(error) = type_node(typed, &resolved, context, &item.node) {
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

fn type_node(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    context: TypeContext,
    node: &AstNode,
) -> Result<Option<CheckedTypeId>, TypecheckError> {
    match node {
        AstNode::Comment { .. } => Ok(None),
        AstNode::Commented { node, .. } => type_node(typed, resolved, context, node),
        AstNode::Literal(literal) => Ok(Some(type_literal(typed, literal)?)),
        AstNode::Identifier { name, syntax_id } => {
            type_identifier_reference(typed, resolved, context, name, *syntax_id)
        }
        AstNode::QualifiedIdentifier { path } => {
            type_qualified_identifier_reference(typed, resolved, context, path)
        }
        _ => {
            for child in node.children() {
                let _ = type_node(typed, resolved, context, child)?;
            }
            Ok(None)
        }
    }
}

fn type_literal(
    typed: &mut TypedProgram,
    literal: &Literal,
) -> Result<CheckedTypeId, TypecheckError> {
    Ok(match literal {
        Literal::Integer(_) => typed.builtin_types().int,
        Literal::Float(_) => typed.builtin_types().float,
        Literal::String(_) => typed.builtin_types().str_,
        Literal::Character(_) => typed.builtin_types().char_,
        Literal::Boolean(_) => typed.builtin_types().bool_,
        Literal::Nil => {
            return Err(TypecheckError::new(
                TypecheckErrorKind::Unsupported,
                "nil literals are not part of the V1 expression typing milestone",
            ));
        }
    })
}

fn type_identifier_reference(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    context: TypeContext,
    name: &str,
    syntax_id: Option<SyntaxNodeId>,
) -> Result<Option<CheckedTypeId>, TypecheckError> {
    let syntax_id = syntax_id.ok_or_else(|| {
        TypecheckError::new(
            TypecheckErrorKind::InvalidInput,
            format!("identifier '{name}' does not retain a syntax id"),
        )
    })?;
    let reference_id =
        find_reference_by_syntax(resolved, syntax_id, ReferenceKind::Identifier, name)?;
    let type_id = type_for_reference(typed, resolved, reference_id, origin_for(resolved, syntax_id))?;
    typed.record_node_type(syntax_id, context.source_unit_id, type_id)?;
    Ok(Some(type_id))
}

fn type_qualified_identifier_reference(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    context: TypeContext,
    path: &QualifiedPath,
) -> Result<Option<CheckedTypeId>, TypecheckError> {
    let syntax_id = path.syntax_id().ok_or_else(|| {
        TypecheckError::new(
            TypecheckErrorKind::InvalidInput,
            format!(
                "qualified identifier '{}' does not retain a syntax id",
                path.joined()
            ),
        )
    })?;
    let reference_id = find_reference_by_syntax(
        resolved,
        syntax_id,
        ReferenceKind::QualifiedIdentifier,
        &path.joined(),
    )?;
    let type_id = type_for_reference(typed, resolved, reference_id, origin_for(resolved, syntax_id))?;
    typed.record_node_type(syntax_id, context.source_unit_id, type_id)?;
    Ok(Some(type_id))
}

fn find_reference_by_syntax(
    resolved: &ResolvedProgram,
    syntax_id: SyntaxNodeId,
    kind: ReferenceKind,
    display_name: &str,
) -> Result<ReferenceId, TypecheckError> {
    resolved
        .references
        .iter_with_ids()
        .find(|(_, reference)| reference.syntax_id == Some(syntax_id) && reference.kind == kind)
        .map(|(reference_id, _)| reference_id)
        .ok_or_else(|| {
            TypecheckError::with_origin(
                TypecheckErrorKind::InvalidInput,
                format!("reference '{display_name}' is missing from resolver output"),
                origin_for(resolved, syntax_id).unwrap_or(SyntaxOrigin {
                    file: None,
                    line: 1,
                    column: 1,
                    length: display_name.len(),
                }),
            )
        })
}

fn type_for_reference(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    reference_id: ReferenceId,
    origin: Option<SyntaxOrigin>,
) -> Result<CheckedTypeId, TypecheckError> {
    let symbol_id = resolved
        .reference(reference_id)
        .and_then(|reference| reference.resolved)
        .ok_or_else(|| {
            TypecheckError::with_origin(
                TypecheckErrorKind::InvalidInput,
                "resolved reference lost its target symbol",
                origin.clone().unwrap_or(SyntaxOrigin {
                    file: None,
                    line: 1,
                    column: 1,
                    length: 1,
                }),
            )
        })?;
    let type_id = symbol_type(typed, symbol_id, origin.clone())?;
    let typed_reference = typed.typed_reference_mut(reference_id).ok_or_else(|| {
        TypecheckError::with_origin(
            TypecheckErrorKind::Internal,
            "typed reference table lost a resolved reference",
            origin.unwrap_or(SyntaxOrigin {
                file: None,
                line: 1,
                column: 1,
                length: 1,
            }),
        )
    })?;
    typed_reference.resolved_type = Some(type_id);
    Ok(type_id)
}

fn symbol_type(
    typed: &TypedProgram,
    symbol_id: SymbolId,
    origin: Option<SyntaxOrigin>,
) -> Result<CheckedTypeId, TypecheckError> {
    typed
        .typed_symbol(symbol_id)
        .and_then(|symbol| symbol.declared_type)
        .ok_or_else(|| {
            TypecheckError::with_origin(
                TypecheckErrorKind::InvalidInput,
                format!("resolved symbol {} does not have a lowered type yet", symbol_id.0),
                origin.unwrap_or(SyntaxOrigin {
                    file: None,
                    line: 1,
                    column: 1,
                    length: 1,
                }),
            )
        })
}

fn origin_for(resolved: &ResolvedProgram, syntax_id: SyntaxNodeId) -> Option<SyntaxOrigin> {
    resolved.syntax_index().origin(syntax_id).cloned()
}

#[cfg(test)]
mod tests {
    use super::type_literal;
    use crate::{BuiltinType, TypedProgram};
    use fol_parser::ast::{AstParser, Literal};
    use fol_resolver::resolve_package;
    use fol_stream::FileStream;

    #[test]
    fn literal_typing_maps_v1_scalar_literals_to_builtin_types() {
        let fixture_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/parser/simple_var.fol");
        let mut stream = FileStream::from_file(fixture_path).expect("Should open expression fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("Expression fixture should parse");
        let resolved = resolve_package(syntax).expect("Expression fixture should resolve");
        let mut typed = TypedProgram::from_resolved(resolved);

        assert_eq!(
            typed.type_table().get(type_literal(&mut typed, &Literal::Integer(1)).unwrap()),
            Some(&crate::CheckedType::Builtin(BuiltinType::Int))
        );
        assert_eq!(
            typed
                .type_table()
                .get(type_literal(&mut typed, &Literal::String("ok".to_string())).unwrap()),
            Some(&crate::CheckedType::Builtin(BuiltinType::Str))
        );
    }
}
