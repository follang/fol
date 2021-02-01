use std::fmt;
pub use crate::syntax::nodes::{NodeTrait, StatTrait};

#[derive(Clone)]
pub enum NodeExprType {
    int,
    flt,
    chr,
    bol,
    arr,
    vec,
    seq,
    mat,
    set,
    map,
    axi,
    tab,
    r#str,
    num,
    ptr,
    err,
    opt,
    nev,
    uni,
    any,
    non,
    nil,
    rec,
    ent,
    blu,
    std,
    loc,
    url,
    blk,
    rut,
    pat,
    gen,
}

impl NodeTrait for NodeExprType {}
impl StatTrait for NodeExprType {}

impl fmt::Display for NodeExprType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            NodeExprType::int => write!(f, "int"),
            NodeExprType::flt => write!(f, "flt"),
            NodeExprType::chr => write!(f, "chr"),
            NodeExprType::bol => write!(f, "bol"),
            NodeExprType::arr => write!(f, "arr"),
            NodeExprType::vec => write!(f, "vec"),
            NodeExprType::seq => write!(f, "seq"),
            NodeExprType::mat => write!(f, "mat"),
            NodeExprType::set => write!(f, "set"),
            NodeExprType::map => write!(f, "map"),
            NodeExprType::axi => write!(f, "axi"),
            NodeExprType::tab => write!(f, "tab"),
            NodeExprType::r#str => write!(f, "str"),
            NodeExprType::num => write!(f, "num"),
            NodeExprType::ptr => write!(f, "ptr"),
            NodeExprType::err => write!(f, "err"),
            NodeExprType::opt => write!(f, "opt"),
            NodeExprType::nev => write!(f, "nev"),
            NodeExprType::uni => write!(f, "uni"),
            NodeExprType::any => write!(f, "any"),
            NodeExprType::non => write!(f, "non"),
            NodeExprType::nil => write!(f, "nil"),
            NodeExprType::rec => write!(f, "rec"),
            NodeExprType::ent => write!(f, "ent"),
            NodeExprType::blu => write!(f, "blu"),
            NodeExprType::std => write!(f, "std"),
            NodeExprType::loc => write!(f, "loc"),
            NodeExprType::url => write!(f, "url"),
            NodeExprType::blk => write!(f, "blk"),
            NodeExprType::rut => write!(f, "rut"),
            NodeExprType::pat => write!(f, "pat"),
            NodeExprType::gen => write!(f, "gen"),
        }
    }
}
