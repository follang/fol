#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TypecheckCapabilityModel {
    Core,
    Alloc,
    #[default]
    Std,
}

impl TypecheckCapabilityModel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Alloc => "alloc",
            Self::Std => "std",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TypecheckConfig {
    pub capability_model: TypecheckCapabilityModel,
}
