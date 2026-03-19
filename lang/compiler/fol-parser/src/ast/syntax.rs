use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SyntaxNodeId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxOrigin {
    pub file: Option<String>,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl SyntaxOrigin {
    pub fn from_token(token: &fol_lexer::lexer::stage3::element::Element) -> Self {
        let loc = token.loc();
        Self {
            file: loc.source().map(|src| src.path(true)),
            line: loc.row(),
            column: loc.col(),
            length: loc.len(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SyntaxIndex {
    origins: Vec<SyntaxOrigin>,
}

impl SyntaxIndex {
    pub fn insert(&mut self, origin: SyntaxOrigin) -> SyntaxNodeId {
        let id = SyntaxNodeId(self.origins.len());
        self.origins.push(origin);
        id
    }

    pub fn origin(&self, id: SyntaxNodeId) -> Option<&SyntaxOrigin> {
        self.origins.get(id.0)
    }

    pub fn len(&self) -> usize {
        self.origins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.origins.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedTopLevel {
    pub node_id: SyntaxNodeId,
    pub node: super::node::AstNode,
    pub meta: ParsedTopLevelMeta,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedSourceUnit {
    pub path: String,
    pub package: String,
    pub namespace: String,
    pub kind: ParsedSourceUnitKind,
    pub items: Vec<ParsedTopLevel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedSourceUnitKind {
    Ordinary,
    Build,
}

impl ParsedSourceUnitKind {
    pub fn from_path(path: &str) -> Self {
        if path == "build.fol" || path.ends_with("/build.fol") {
            Self::Build
        } else {
            Self::Ordinary
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedPackage {
    pub package: String,
    pub source_units: Vec<ParsedSourceUnit>,
    pub syntax_index: SyntaxIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedDeclVisibility {
    Normal,
    Exported,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedDeclScope {
    Package,
    Namespace,
    File,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ParsedTopLevelMeta {
    pub visibility: Option<ParsedDeclVisibility>,
    pub scope: Option<ParsedDeclScope>,
}

impl ParsedTopLevel {
    pub fn declaration_visibility(&self) -> Option<ParsedDeclVisibility> {
        self.meta.visibility
    }

    pub fn declaration_scope(&self) -> Option<ParsedDeclScope> {
        self.meta.scope
    }
}

impl ParsedPackage {
    pub fn from_sources_and_entries(
        sources: &[fol_stream::Source],
        entries: Vec<ParsedTopLevel>,
        syntax_index: SyntaxIndex,
    ) -> Self {
        let package = sources
            .first()
            .map(|source| source.package.clone())
            .unwrap_or_default();
        let mut source_units = sources
            .iter()
            .map(|source| ParsedSourceUnit {
                path: source.path.clone(),
                package: source.package.clone(),
                namespace: source.namespace.clone(),
                kind: ParsedSourceUnitKind::from_path(&source.path),
                items: Vec::new(),
            })
            .collect::<Vec<_>>();
        let mut path_to_index = HashMap::new();

        for (index, source) in sources.iter().enumerate() {
            path_to_index.insert(source.path.clone(), index);
        }

        for mut entry in entries {
            let unit_index = syntax_index
                .origin(entry.node_id)
                .and_then(|origin| origin.file.as_ref())
                .and_then(|file| path_to_index.get(file))
                .copied()
                .unwrap_or(0);

            if let Some(unit) = source_units.get_mut(unit_index) {
                entry.meta = ParsedTopLevelMeta {
                    visibility: entry.node.declaration_visibility(),
                    scope: entry.node.declaration_scope(&unit.package, &unit.namespace),
                };
                unit.items.push(entry);
            }
        }

        Self {
            package,
            source_units,
            syntax_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ParsedSourceUnitKind;

    #[test]
    fn parsed_source_unit_kind_distinguishes_build_paths() {
        assert_eq!(
            ParsedSourceUnitKind::from_path("src/main.fol"),
            ParsedSourceUnitKind::Ordinary
        );
        assert_eq!(
            ParsedSourceUnitKind::from_path("build.fol"),
            ParsedSourceUnitKind::Build
        );
        assert_eq!(
            ParsedSourceUnitKind::from_path("/tmp/pkg/build.fol"),
            ParsedSourceUnitKind::Build
        );
    }
}
