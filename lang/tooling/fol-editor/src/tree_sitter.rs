#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeSitterCorpusCase {
    pub name: &'static str,
    pub source: &'static str,
}

const GRAMMAR_SOURCE: &str = include_str!("../tree-sitter/grammar.js");
const HIGHLIGHTS_QUERY: &str = include_str!("../queries/fol/highlights.scm");
const CORPUS_DECLARATIONS: &str = include_str!("../tree-sitter/test/corpus/declarations.txt");
const CORPUS_EXPRESSIONS: &str = include_str!("../tree-sitter/test/corpus/expressions.txt");
const CORPUS_RECOVERABLE: &str = include_str!("../tree-sitter/test/corpus/recoverable.txt");

pub fn fol_tree_sitter_grammar() -> &'static str {
    GRAMMAR_SOURCE
}

pub fn fol_tree_sitter_highlights_query() -> &'static str {
    HIGHLIGHTS_QUERY
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
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        fol_tree_sitter_corpus, fol_tree_sitter_grammar, fol_tree_sitter_highlights_query,
    };

    #[test]
    fn grammar_scaffold_has_the_fol_language_name() {
        let grammar = fol_tree_sitter_grammar();
        assert!(grammar.contains("name: 'fol'"));
        assert!(grammar.contains("$.source_file"));
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
            "@keyword",
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
    fn corpus_smoke_cases_cover_real_v1_surfaces() {
        let corpus = fol_tree_sitter_corpus();
        assert_eq!(corpus.len(), 3);
        assert!(corpus.iter().any(|case| case.source.contains("use shared: loc")));
        assert!(corpus.iter().any(|case| case.source.contains("when(flag)")));
        assert!(corpus.iter().any(|case| case.source.contains("report \"bad-input\"")));
    }
}
