#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TypecheckCapabilityModel {
    Core,
    Alloc,
    #[default]
    Std,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TypecheckConfig {
    pub capability_model: TypecheckCapabilityModel,
}
