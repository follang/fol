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
            "@type",
            "@function",
            "@method",
            "@variable",
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
        assert!(query.contains("(decl_modifiers (identifier) @attribute)"));
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
}
