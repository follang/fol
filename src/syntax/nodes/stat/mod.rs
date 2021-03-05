pub mod datatype;
pub mod ident;
pub mod decshort;
pub mod declong;
pub use crate::syntax::nodes::stat::{
        declong::NodeStatDecL,
        decshort::NodeStatDecS,
        datatype::NodeStatDatatypes,
        ident::NodeStatIdent,
};
