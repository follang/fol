pub mod datatype;
pub mod ident;
pub mod decshort;
pub mod declong;
pub mod conloop;
pub mod conwhen;
pub use crate::syntax::nodes::stat::{
        declong::NodeStatDecL,
        decshort::NodeStatDecS,
        datatype::NodeStatDatatypes,
        ident::NodeStatIdent,
        conloop::NodeStatLoop,
        conwhen::NodeStatWhen,
};
