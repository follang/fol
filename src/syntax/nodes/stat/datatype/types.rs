use std::fmt;
pub use crate::syntax::nodes::{Node, Stat};

#[derive(Clone)]
pub enum Datatype {
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

impl Node for Datatype {}
impl Stat for Datatype {}

impl fmt::Display for Datatype {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Datatype::int => write!(f, "int"),
            Datatype::flt => write!(f, "flt"),
            Datatype::chr => write!(f, "chr"),
            Datatype::bol => write!(f, "bol"),
            Datatype::arr => write!(f, "arr"),
            Datatype::vec => write!(f, "vec"),
            Datatype::seq => write!(f, "seq"),
            Datatype::mat => write!(f, "mat"),
            Datatype::set => write!(f, "set"),
            Datatype::map => write!(f, "map"),
            Datatype::axi => write!(f, "axi"),
            Datatype::tab => write!(f, "tab"),
            Datatype::r#str => write!(f, "str"),
            Datatype::num => write!(f, "num"),
            Datatype::ptr => write!(f, "ptr"),
            Datatype::err => write!(f, "err"),
            Datatype::opt => write!(f, "opt"),
            Datatype::nev => write!(f, "nev"),
            Datatype::uni => write!(f, "uni"),
            Datatype::any => write!(f, "any"),
            Datatype::non => write!(f, "non"),
            Datatype::nil => write!(f, "nil"),
            Datatype::rec => write!(f, "rec"),
            Datatype::ent => write!(f, "ent"),
            Datatype::blu => write!(f, "blu"),
            Datatype::std => write!(f, "std"),
            Datatype::loc => write!(f, "loc"),
            Datatype::url => write!(f, "url"),
            Datatype::blk => write!(f, "blk"),
            Datatype::rut => write!(f, "rut"),
            Datatype::pat => write!(f, "pat"),
            Datatype::gen => write!(f, "gen"),
        }
    }
}
