use crate::{ids::LoweredTypeId, LoweredRecoverableAbi, LoweredWorkspace};
use std::fmt::Write;

pub fn render_lowered_workspace(workspace: &LoweredWorkspace) -> String {
    let mut output = String::new();
    let _ = writeln!(
        output,
        "workspace entry={} packages={} types={}",
        workspace.entry_identity().display_name,
        workspace.package_count(),
        workspace.type_table().len()
    );
    match workspace.recoverable_abi() {
        LoweredRecoverableAbi::TaggedResultObject {
            tag_type,
            success_tag,
            error_tag,
            success_slot,
            error_slot,
        } => {
            let _ = writeln!(
                output,
                "recoverable-abi kind=tagged-result-object tag=t{} success-tag={} error-tag={} success-slot={} error-slot={}",
                tag_type.0,
                success_tag,
                error_tag,
                success_slot,
                error_slot
            );
        }
    }
    for type_index in 0..workspace.type_table().len() {
        let type_id = LoweredTypeId(type_index);
        if let Some(ty) = workspace.type_table().get(type_id) {
            let _ = writeln!(output, "type t{} = {:?}", type_id.0, ty);
        }
    }
    for package in workspace.packages() {
        let _ = writeln!(
            output,
            "package {} id={} kind={:?} root={}",
            package.identity.display_name,
            package.id.0,
            package.identity.source_kind,
            package.identity.canonical_root
        );
        for export in &package.exports {
            let _ = writeln!(
                output,
                "  export source={} mounted={}",
                export.source_namespace,
                export.mounted_namespace_suffix.as_deref().unwrap_or("<root>")
            );
        }
        for source_unit in &package.source_units {
            let _ = writeln!(
                output,
                "  source {} path={} namespace={}",
                source_unit.source_unit_id.0,
                source_unit.path,
                source_unit.namespace
            );
        }
        for type_decl in package.type_decls.values() {
            let _ = writeln!(
                output,
                "  type-decl {} symbol={} runtime=t{} kind={:?}",
                type_decl.name,
                type_decl.symbol_id.0,
                type_decl.runtime_type.0,
                type_decl.kind
            );
        }
        for global in package.global_decls.values() {
            let _ = writeln!(
                output,
                "  global g{} {} symbol={} type=t{} mutable={}",
                global.id.0,
                global.name,
                global.symbol_id.0,
                global.type_id.0,
                global.mutable
            );
        }
        for routine in package.routine_decls.values() {
            let _ = writeln!(
                output,
                "  routine r{} {} symbol={:?} signature={:?} receiver={:?} entry=b{} body={:?}",
                routine.id.0,
                routine.name,
                routine.symbol_id.map(|id| id.0),
                routine.signature.map(|id| format!("t{}", id.0)),
                routine.receiver_type.map(|id| format!("t{}", id.0)),
                routine.entry_block.0,
                routine.body_result.map(|id| format!("l{}", id.0))
            );
            let _ = writeln!(
                output,
                "    params [{}]",
                routine
                    .params
                    .iter()
                    .map(|local| format!("l{}", local.0))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            for (local_id, local) in routine.locals.iter_with_ids() {
                let _ = writeln!(
                    output,
                    "    local l{} name={} type={}",
                    local_id.0,
                    local.name.as_deref().unwrap_or("_"),
                    local
                        .type_id
                        .map(|type_id| format!("t{}", type_id.0))
                        .unwrap_or_else(|| "_".to_string())
                );
            }
            for (block_id, block) in routine.blocks.iter_with_ids() {
                let _ = writeln!(output, "    block b{}", block_id.0);
                for instr_id in &block.instructions {
                    if let Some(instr) = routine.instructions.get(*instr_id) {
                        let _ = writeln!(
                            output,
                            "      i{} result={} {:?}",
                            instr.id.0,
                            instr
                                .result
                                .map(|local| format!("l{}", local.0))
                                .unwrap_or_else(|| "_".to_string()),
                            instr.kind
                        );
                    }
                }
                let _ = writeln!(output, "      term {:?}", block.terminator);
            }
        }
    }
    if !workspace.entry_candidates().is_empty() {
        let _ = writeln!(output, "entry-candidates");
        for entry in workspace.entry_candidates() {
            let _ = writeln!(
                output,
                "  {}::{} -> r{}",
                entry.package_identity.display_name,
                entry.name,
                entry.routine_id.0
            );
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::render_lowered_workspace;
    use crate::Lowerer;
    use fol_parser::ast::AstParser;
    use fol_resolver::resolve_workspace;
    use fol_stream::FileStream;
    use fol_typecheck::Typechecker;

    #[test]
    fn lowered_workspace_snapshot_is_stable_and_human_readable() {
        let fixture_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/parser/simple_var.fol");
        let mut stream = FileStream::from_file(fixture_path).expect("Should open lowering fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("snapshot fixture should parse");
        let resolved = resolve_workspace(syntax).expect("snapshot fixture should resolve");
        let typed = Typechecker::new()
            .check_resolved_workspace(resolved)
            .expect("snapshot fixture should typecheck");
        let lowered = Lowerer::new()
            .lower_typed_workspace(typed)
            .expect("snapshot fixture should lower");

        let first = render_lowered_workspace(&lowered);
        let second = render_lowered_workspace(&lowered);

        assert_eq!(first, second);
        assert!(first.contains("workspace entry=parser"));
        assert!(first.contains("recoverable-abi kind=tagged-result-object"));
        assert!(first.contains("package parser"));
        assert!(first.contains("routine"));
        assert!(first.contains("block b0"));
    }
}
