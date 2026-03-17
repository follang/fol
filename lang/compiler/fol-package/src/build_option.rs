#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardTargetDeclaration {
    pub name: String,
    pub default: Option<BuildTargetTriple>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildTargetArch {
    X86_64,
    Aarch64,
}

impl BuildTargetArch {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64",
            Self::Aarch64 => "aarch64",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildTargetOs {
    Linux,
    Macos,
    Windows,
}

impl BuildTargetOs {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Linux => "linux",
            Self::Macos => "macos",
            Self::Windows => "windows",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildTargetEnvironment {
    Gnu,
    Musl,
    Msvc,
}

impl BuildTargetEnvironment {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gnu => "gnu",
            Self::Musl => "musl",
            Self::Msvc => "msvc",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildTargetTriple {
    pub arch: BuildTargetArch,
    pub os: BuildTargetOs,
    pub environment: BuildTargetEnvironment,
}

impl BuildTargetTriple {
    pub fn parse(raw: &str) -> Option<Self> {
        let mut parts = raw.split('-');
        let arch = match parts.next()? {
            "x86_64" => BuildTargetArch::X86_64,
            "aarch64" => BuildTargetArch::Aarch64,
            _ => return None,
        };
        let os = match parts.next()? {
            "linux" => BuildTargetOs::Linux,
            "macos" => BuildTargetOs::Macos,
            "windows" => BuildTargetOs::Windows,
            _ => return None,
        };
        let environment = match parts.next()? {
            "gnu" => BuildTargetEnvironment::Gnu,
            "musl" => BuildTargetEnvironment::Musl,
            "msvc" => BuildTargetEnvironment::Msvc,
            _ => return None,
        };
        if parts.next().is_some() {
            return None;
        }
        Some(Self {
            arch,
            os,
            environment,
        })
    }

    pub fn render(&self) -> String {
        format!(
            "{}-{}-{}",
            self.arch.as_str(),
            self.os.as_str(),
            self.environment.as_str()
        )
    }
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
        BuildOptionDeclaration, BuildOptionDeclarationSet, BuildTargetArch,
        BuildTargetEnvironment, BuildTargetOs, BuildTargetTriple,
        StandardOptimizeDeclaration, StandardTargetDeclaration, UserOptionDeclaration,
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
                default: Some(BuildTargetTriple {
                    arch: BuildTargetArch::X86_64,
                    os: BuildTargetOs::Linux,
                    environment: BuildTargetEnvironment::Gnu,
                }),
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

    #[test]
    fn build_target_triple_parses_and_renders_canonical_triplets() {
        let triple = BuildTargetTriple::parse("x86_64-linux-gnu")
            .expect("canonical triples should parse");

        assert_eq!(triple.arch, BuildTargetArch::X86_64);
        assert_eq!(triple.os, BuildTargetOs::Linux);
        assert_eq!(triple.environment, BuildTargetEnvironment::Gnu);
        assert_eq!(triple.render(), "x86_64-linux-gnu");
    }

    #[test]
    fn build_target_triple_rejects_unknown_or_malformed_triplets() {
        assert!(BuildTargetTriple::parse("x86_64").is_none());
        assert!(BuildTargetTriple::parse("sparc-linux-gnu").is_none());
        assert!(BuildTargetTriple::parse("x86_64-linux-gnu-extra").is_none());
    }
}
