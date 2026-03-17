#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeSitterCorpusCase {
    pub name: &'static str,
    pub source: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeSitterQuerySnapshot {
    pub name: &'static str,
    pub query: &'static str,
}

const GRAMMAR_SOURCE: &str = include_str!("../tree-sitter/grammar.js");
const TREE_SITTER_CONFIG: &str = include_str!("../tree-sitter/tree-sitter.json");
const HIGHLIGHTS_QUERY: &str = include_str!("../queries/fol/highlights.scm");
const LOCALS_QUERY: &str = include_str!("../queries/fol/locals.scm");
const SYMBOLS_QUERY: &str = include_str!("../queries/fol/symbols.scm");
const CORPUS_DECLARATIONS: &str = include_str!("../tree-sitter/test/corpus/declarations.txt");
const CORPUS_EXPRESSIONS: &str = include_str!("../tree-sitter/test/corpus/expressions.txt");
const CORPUS_RECOVERABLE: &str = include_str!("../tree-sitter/test/corpus/recoverable.txt");
const CORPUS_SHOWCASE: &str =
    include_str!("../../../../test/apps/showcases/full_v1_showcase/app/main.fol");

pub fn fol_tree_sitter_grammar() -> &'static str {
    GRAMMAR_SOURCE
}

pub fn fol_tree_sitter_config() -> &'static str {
    TREE_SITTER_CONFIG
}

pub fn fol_tree_sitter_highlights_query() -> &'static str {
    HIGHLIGHTS_QUERY
}

pub fn fol_tree_sitter_locals_query() -> &'static str {
    LOCALS_QUERY
}

pub fn fol_tree_sitter_symbols_query() -> &'static str {
    SYMBOLS_QUERY
}

pub fn fol_tree_sitter_corpus() -> &'static [TreeSitterCorpusCase] {
    &[
        TreeSitterCorpusCase {
            name: "declarations",
            source: CORPUS_DECLARATIONS,
        },
        TreeSitterCorpusCase {
            name: "expressions",
            source: CORPUS_EXPRESSIONS,
        },
        TreeSitterCorpusCase {
            name: "recoverable",
            source: CORPUS_RECOVERABLE,
        },
        TreeSitterCorpusCase {
            name: "showcase",
            source: CORPUS_SHOWCASE,
        },
    ]
}

pub fn fol_tree_sitter_query_snapshots() -> &'static [TreeSitterQuerySnapshot] {
    &[
        TreeSitterQuerySnapshot {
            name: "highlights",
            query: HIGHLIGHTS_QUERY,
        },
        TreeSitterQuerySnapshot {
            name: "locals",
            query: LOCALS_QUERY,
        },
        TreeSitterQuerySnapshot {
            name: "symbols",
            query: SYMBOLS_QUERY,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        fol_tree_sitter_config, fol_tree_sitter_corpus, fol_tree_sitter_grammar, fol_tree_sitter_highlights_query,
        fol_tree_sitter_locals_query, fol_tree_sitter_query_snapshots,
        fol_tree_sitter_symbols_query,
    };
    use std::path::{Path, PathBuf};
    use std::process::Command;

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "fol_editor_tree_query_{}_{}_{}",
            label,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time should be after epoch")
                .as_nanos()
        ))
    }

    fn build_bundle_root(label: &str) -> PathBuf {
        let root = temp_root(label);
        crate::editor_tree_generate_bundle(&root).expect("tree bundle generation should succeed");
        root
    }

    fn tree_sitter_cache_root(label: &str) -> PathBuf {
        let root = temp_root(&format!("cache_{label}"));
        std::fs::create_dir_all(&root).expect("tree-sitter cache root should be created");
        root
    }

    fn run_tree_sitter_query(bundle_root: &Path, query_path: &Path, source_path: &Path) -> std::process::Output {
        let cache_root = tree_sitter_cache_root("query");
        Command::new("tree-sitter")
            .env("XDG_CACHE_HOME", &cache_root)
            .arg("query")
            .arg("--grammar-path")
            .arg(bundle_root)
            .arg(query_path)
            .arg(source_path)
            .output()
            .expect("tree-sitter query should run")
    }

    #[test]
    fn grammar_scaffold_has_the_fol_language_name() {
        let grammar = fol_tree_sitter_grammar();
        assert!(grammar.contains("name: 'fol'"));
        assert!(grammar.contains("$.source_file"));
    }

    #[test]
    fn tree_sitter_config_declares_fol_scope_and_queries() {
        let config = fol_tree_sitter_config();
        assert!(config.contains("\"scope\": \"source.fol\""));
        assert!(config.contains("\"file-types\": [\"fol\"]"));
        assert!(config.contains("\"highlights\": \"queries/fol/highlights.scm\""));
        assert!(config.contains("\"locals\": \"queries/fol/locals.scm\""));
    }

    #[test]
    fn grammar_covers_lexical_tokens_declarations_and_control_flow() {
        let grammar = fol_tree_sitter_grammar();
        for needle in [
            "identifier",
            "integer_literal",
            "string_literal",
            "comment",
            "doc_comment",
            "use_decl",
            "var_decl",
            "fun_decl",
            "log_decl",
            "typ_decl",
            "ali_decl",
            "block",
            "when_expr",
            "loop_expr",
            "return_stmt",
            "report_stmt",
            "panic_stmt",
            "unreachable_stmt",
            "break_stmt",
        ] {
            assert!(grammar.contains(needle), "missing grammar rule marker: {needle}");
        }
    }

    #[test]
    fn grammar_covers_v1_surface_families_explicitly() {
        let grammar = fol_tree_sitter_grammar();
        for needle in [
            "source_kind",
            "'loc'",
            "'std'",
            "'pkg'",
            "decl_modifiers",
            "modifier_list",
            "typed_binding",
            "method_decl",
            "record_type",
            "entry_type",
            "qualified_path",
            "dot_intrinsic",
            "check_expr",
            "container_type",
            "shell_type",
            "nil_literal",
            "unwrap_expr",
        ] {
            assert!(grammar.contains(needle), "missing v1 grammar marker: {needle}");
        }
    }

    #[test]
    fn grammar_mentions_editor_friendly_recovery_shapes() {
        let grammar = fol_tree_sitter_grammar();
        assert!(grammar.contains("conflicts: $ => ["));
        assert!(grammar.contains("extras: $ => ["));
        assert!(grammar.contains("optional($.error_type)"));
    }

    #[test]
    fn highlight_query_covers_declarations_keywords_and_literals() {
        let query = fol_tree_sitter_highlights_query();
        for needle in [
            "@keyword.import",
            "@keyword.function",
            "@keyword.type",
            "@keyword.conditional",
            "@keyword.return",
            "@keyword.exception",
            "@type",
            "@function",
            "@method",
            "@variable",
            "@operator",
            "@constant.builtin",
            "@string",
            "@number",
            "@comment",
        ] {
            assert!(query.contains(needle), "missing highlight capture: {needle}");
        }
    }

    #[test]
    fn highlight_query_covers_intrinsics_and_qualified_paths() {
        let query = fol_tree_sitter_highlights_query();
        assert!(query.contains("(dot_intrinsic name: (identifier) @function.builtin"));
        assert!(query.contains("(qualified_path"));
        assert!(query.contains("@namespace"));
    }

    #[test]
    fn grammar_and_query_cover_bracketed_declaration_modifiers() {
        let grammar = fol_tree_sitter_grammar();
        let query = fol_tree_sitter_highlights_query();

        assert!(grammar.contains("optional(field('modifiers', $.decl_modifiers))"));
        assert!(grammar.contains("seq('[', optional($.modifier_list), ']')"));
        assert!(query.contains("(decl_modifiers (modifier_list (identifier) @attribute))"));
    }

    #[test]
    fn highlight_query_captures_each_declaration_head_explicitly() {
        let query = fol_tree_sitter_highlights_query();
        for needle in [
            "(use_decl \"use\" @keyword.import)",
            "(var_decl \"var\" @keyword)",
            "(fun_decl \"fun\" @keyword.function)",
            "(log_decl \"log\" @keyword.function)",
            "(typ_decl \"typ\" @keyword.type)",
            "(ali_decl \"ali\" @keyword.type)",
            "(when_expr \"when\" @keyword.conditional)",
            "(loop_expr \"loop\" @keyword.repeat)",
            "(return_stmt \"return\" @keyword.return)",
            "(report_stmt \"report\" @keyword.exception)",
            "(panic_stmt \"panic\" @keyword.exception)",
            "(unreachable_stmt) @keyword.exception",
            "(check_expr \"check\" @keyword.exception)",
            "(break_stmt \"break\" @keyword.repeat)",
        ] {
            assert!(query.contains(needle), "missing declaration head capture: {needle}");
        }
    }

    #[test]
    fn highlight_query_distinguishes_declaration_names_by_role() {
        let query = fol_tree_sitter_highlights_query();
        for needle in [
            "(use_decl name: (identifier) @namespace)",
            "(typ_decl name: (identifier) @type)",
            "(ali_decl name: (identifier) @type.definition)",
            "(error_type \"/\" @operator)",
            "(fun_decl declaration: (plain_fun_decl name: (identifier) @function))",
            "(fun_decl declaration: (method_decl name: (identifier) @function.method))",
            "(log_decl declaration: (plain_log_decl name: (identifier) @function.builtin))",
            "(var_decl (typed_binding name: (identifier) @variable))",
            "(unwrap_expr \"!\" @operator)",
            "(nil_literal) @constant.builtin",
        ] {
            assert!(query.contains(needle), "missing declaration role capture: {needle}");
        }
    }

    #[test]
    fn highlight_query_uses_current_declaration_field_shapes() {
        let grammar = fol_tree_sitter_grammar();
        let query = fol_tree_sitter_highlights_query();

        assert!(grammar.contains("field('declaration', choice($.plain_fun_decl, $.method_decl))"));
        assert!(grammar.contains("field('declaration', choice($.plain_log_decl, $.method_decl))"));
        assert!(grammar.contains("seq('var', $.typed_binding"));

        for needle in [
            "(use_decl \"use\" @keyword.import)",
            "(fun_decl \"fun\" @keyword.function)",
            "(log_decl \"log\" @keyword.function)",
            "(typ_decl \"typ\" @keyword.type)",
            "(ali_decl \"ali\" @keyword.type)",
            "(use_decl name: (identifier) @namespace)",
            "(ali_decl name: (identifier) @type.definition)",
            "(fun_decl declaration: (plain_fun_decl",
            "(fun_decl declaration: (method_decl",
            "(log_decl declaration: (plain_log_decl",
            "(log_decl declaration: (method_decl",
            "(var_decl (typed_binding name: (identifier) @variable))",
        ] {
            assert!(
                query.contains(needle),
                "highlight query drifted away from the current grammar shape: {needle}"
            );
        }
    }

    #[test]
    fn locals_query_captures_bindings_parameters_and_function_names() {
        let query = fol_tree_sitter_locals_query();
        for needle in [
            "@local.scope",
            "@local.definition",
            "@local.reference",
            "(param name: (identifier) @local.definition)",
            "(var_decl (typed_binding name: (identifier) @local.definition))",
            "(fun_decl name: (identifier) @local.definition.function)",
        ] {
            assert!(query.contains(needle), "missing locals capture marker: {needle}");
        }
    }

    #[test]
    fn symbols_query_captures_types_functions_bindings_and_namespaces() {
        let query = fol_tree_sitter_symbols_query();
        for needle in [
            "@symbol.scope",
            "@symbol.function",
            "@symbol.type",
            "@symbol.variable",
            "@symbol.namespace",
        ] {
            assert!(query.contains(needle), "missing symbol capture marker: {needle}");
        }
    }

    #[test]
    fn query_snapshots_stay_in_editor_consumable_order() {
        let snapshots = fol_tree_sitter_query_snapshots();
        assert_eq!(snapshots.len(), 3);
        assert_eq!(snapshots[0].name, "highlights");
        assert_eq!(snapshots[1].name, "locals");
        assert_eq!(snapshots[2].name, "symbols");
    }

    #[test]
    fn corpus_smoke_cases_cover_real_v1_surfaces() {
        let corpus = fol_tree_sitter_corpus();
        assert_eq!(corpus.len(), 4);
        assert!(corpus.iter().any(|case| case.source.contains("use shared: loc")));
        assert!(corpus.iter().any(|case| case.source.contains("when(flag)")));
        assert!(corpus.iter().any(|case| case.source.contains("report \"bad-input\"")));
        assert!(corpus.iter().any(|case| case.source.contains("typ Summary: rec")));
    }

    #[test]
    fn generated_bundle_highlight_query_validates_against_tree_sitter() {
        let root = build_bundle_root("valid");
        let output = run_tree_sitter_query(
            &root,
            &root.join("queries/fol/highlights.scm"),
            &PathBuf::from("xtra/logtiny/src/log.fol"),
        );

        assert!(
            output.status.success(),
            "tree-sitter query failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("function"));
        assert!(stdout.contains("attribute"));

        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn invalid_highlight_query_node_references_fail_bundle_validation() {
        let root = build_bundle_root("invalid");
        let query_path = root.join("queries/fol/highlights.scm");
        std::fs::write(&query_path, "(missing_fol_node) @keyword").unwrap();

        let output = run_tree_sitter_query(&root, &query_path, &PathBuf::from("xtra/logtiny/src/log.fol"));

        assert!(
            !output.status.success(),
            "invalid query unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("Invalid node type")
                || String::from_utf8_lossy(&output.stderr).contains("Query error"),
            "unexpected error output:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn declaration_heavy_real_fixtures_keep_highlight_captures_stable() {
        let root = build_bundle_root("declaration_snapshots");
        let logtiny_output = run_tree_sitter_query(
            &root,
            &root.join("queries/fol/highlights.scm"),
            &PathBuf::from("xtra/logtiny/src/log.fol"),
        );
        assert!(logtiny_output.status.success());
        let logtiny = String::from_utf8_lossy(&logtiny_output.stdout);
        for needle in [
            "keyword.type",
            "type.definition",
            "attribute",
            "function",
            "function.builtin",
            "variable.parameter",
        ] {
            assert!(
                logtiny.contains(needle),
                "declaration-heavy package fixture lost highlight capture: {needle}\n{logtiny}"
            );
        }

        let showcase_output = run_tree_sitter_query(
            &root,
            &root.join("queries/fol/highlights.scm"),
            &PathBuf::from("test/apps/showcases/full_v1_showcase/app/main.fol"),
        );
        assert!(showcase_output.status.success());
        let showcase = String::from_utf8_lossy(&showcase_output.stdout);
        for needle in [
            "keyword.function",
            "type",
            "namespace",
            "variable",
            "property",
        ] {
            assert!(
                showcase.contains(needle),
                "showcase fixture lost declaration highlight capture: {needle}\n{showcase}"
            );
        }

        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn control_and_effect_keywords_stay_highlighted_in_real_fixtures() {
        let root = build_bundle_root("keyword_effects");
        let output = run_tree_sitter_query(
            &root,
            &root.join("queries/fol/highlights.scm"),
            &PathBuf::from("test/apps/fixtures/intrinsics_panic_check/main.fol"),
        );
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        for needle in [
            "keyword.conditional",
            "keyword.return",
            "keyword.exception",
            "keyword.repeat",
        ] {
            assert!(
                stdout.contains(needle),
                "control/effect fixture lost keyword capture: {needle}\n{stdout}"
            );
        }

        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn import_source_kinds_keep_distinct_keyword_captures() {
        let query = fol_tree_sitter_highlights_query();
        for needle in [
            "(use_decl source_kind: (source_kind \"loc\" @keyword.import))",
            "(use_decl source_kind: (source_kind \"pkg\" @keyword.import))",
            "(use_decl source_kind: (source_kind \"std\" @keyword.import))",
        ] {
            assert!(query.contains(needle), "missing source-kind capture: {needle}");
        }

        let root = build_bundle_root("import_source_kinds");
        let output = run_tree_sitter_query(
            &root,
            &root.join("queries/fol/highlights.scm"),
            &PathBuf::from("test/apps/fixtures/mixed_loc_std_pkg/app/main.fol"),
        );
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        for needle in ["loc", "std", "pkg", "keyword.import"] {
            assert!(
                stdout.contains(needle),
                "mixed import fixture lost source-kind capture: {needle}\n{stdout}"
            );
        }

        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn shell_surfaces_keep_nil_and_boundary_captures() {
        let root = build_bundle_root("shell_surfaces");
        let optional_output = run_tree_sitter_query(
            &root,
            &root.join("queries/fol/highlights.scm"),
            &PathBuf::from("test/apps/fixtures/shell_optional/main.fol"),
        );
        assert!(optional_output.status.success());
        let optional = String::from_utf8_lossy(&optional_output.stdout);
        for needle in ["constant.builtin", "operator"] {
            assert!(
                optional.contains(needle),
                "optional shell fixture lost shell capture: {needle}\n{optional}"
            );
        }

        let boundary_output = run_tree_sitter_query(
            &root,
            &root.join("queries/fol/highlights.scm"),
            &PathBuf::from("test/apps/fixtures/shell_vs_recoverable_boundary/main.fol"),
        );
        assert!(boundary_output.status.success());
        let boundary = String::from_utf8_lossy(&boundary_output.stdout);
        for needle in ["constant.builtin", "operator"] {
            assert!(
                boundary.contains(needle),
                "recoverable boundary fixture lost shell capture: {needle}\n{boundary}"
            );
        }

        std::fs::remove_dir_all(root).ok();
    }
}
