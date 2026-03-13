#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PackageConfig {
    pub std_root: Option<String>,
    pub package_store_root: Option<String>,
    pub package_cache_root: Option<String>,
}
