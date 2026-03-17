macro_rules! define_graph_id {
    ($name:ident, $label:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(pub usize);

        impl $name {
            pub fn index(self) -> usize {
                self.0
            }

            pub fn from_index(index: usize) -> Self {
                Self(index)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}{}", $label, self.0)
            }
        }
    };
}

define_graph_id!(BuildStepId, "step:");
define_graph_id!(BuildArtifactId, "artifact:");
define_graph_id!(BuildModuleId, "module:");
define_graph_id!(BuildGeneratedFileId, "generated:");
define_graph_id!(BuildOptionId, "option:");
define_graph_id!(BuildInstallId, "install:");

#[cfg(test)]
mod tests {
    use super::{
        BuildArtifactId, BuildGeneratedFileId, BuildInstallId, BuildModuleId, BuildOptionId,
        BuildStepId,
    };

    #[test]
    fn build_graph_ids_round_trip_their_raw_indexes() {
        assert_eq!(BuildStepId::from_index(3).index(), 3);
        assert_eq!(BuildArtifactId::from_index(5).index(), 5);
        assert_eq!(BuildModuleId::from_index(7).index(), 7);
        assert_eq!(BuildGeneratedFileId::from_index(11).index(), 11);
        assert_eq!(BuildOptionId::from_index(13).index(), 13);
        assert_eq!(BuildInstallId::from_index(17).index(), 17);
    }

    #[test]
    fn build_graph_ids_render_with_stable_family_prefixes() {
        assert_eq!(BuildStepId(0).to_string(), "step:0");
        assert_eq!(BuildArtifactId(1).to_string(), "artifact:1");
        assert_eq!(BuildModuleId(2).to_string(), "module:2");
        assert_eq!(BuildGeneratedFileId(3).to_string(), "generated:3");
        assert_eq!(BuildOptionId(4).to_string(), "option:4");
        assert_eq!(BuildInstallId(5).to_string(), "install:5");
    }
}
