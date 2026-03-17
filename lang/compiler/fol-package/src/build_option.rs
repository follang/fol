#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardTargetDeclaration {
    pub name: String,
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardOptimizeDeclaration {
    pub name: String,
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildOptionDeclaration {
    StandardTarget(StandardTargetDeclaration),
    StandardOptimize(StandardOptimizeDeclaration),
    User(UserOptionDeclaration),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserOptionDeclaration {
    pub name: String,
    pub help: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildOptionDeclarationSet {
    declarations: Vec<BuildOptionDeclaration>,
}

impl BuildOptionDeclarationSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn declarations(&self) -> &[BuildOptionDeclaration] {
        &self.declarations
    }

    pub fn add(&mut self, declaration: BuildOptionDeclaration) {
        self.declarations.push(declaration);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BuildOptionDeclaration, BuildOptionDeclarationSet, StandardOptimizeDeclaration,
        StandardTargetDeclaration, UserOptionDeclaration,
    };

    #[test]
    fn build_option_declaration_set_starts_empty() {
        let set = BuildOptionDeclarationSet::new();

        assert!(set.declarations().is_empty());
    }

    #[test]
    fn build_option_declaration_set_preserves_inserted_shell_declarations() {
        let mut set = BuildOptionDeclarationSet::new();
        set.add(BuildOptionDeclaration::StandardTarget(
            StandardTargetDeclaration {
                name: "target".to_string(),
                default: Some("x86_64-linux-gnu".to_string()),
            },
        ));
        set.add(BuildOptionDeclaration::StandardOptimize(
            StandardOptimizeDeclaration {
                name: "optimize".to_string(),
                default: Some("debug".to_string()),
            },
        ));
        set.add(BuildOptionDeclaration::User(UserOptionDeclaration {
            name: "docs".to_string(),
            help: Some("enable docs generation".to_string()),
        }));

        assert_eq!(set.declarations().len(), 3);
        assert!(matches!(
            &set.declarations()[0],
            BuildOptionDeclaration::StandardTarget(StandardTargetDeclaration { name, .. })
            if name == "target"
        ));
        assert!(matches!(
            &set.declarations()[1],
            BuildOptionDeclaration::StandardOptimize(StandardOptimizeDeclaration { name, .. })
            if name == "optimize"
        ));
        assert!(matches!(
            &set.declarations()[2],
            BuildOptionDeclaration::User(UserOptionDeclaration { name, .. })
            if name == "docs"
        ));
    }
}
