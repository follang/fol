use std::fmt;
use crate::syntax::nodes::{NodeTrait, OptsTrait};

#[derive(Clone)]
pub enum TypOpts {
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

impl NodeTrait for TypOpts {}
impl OptsTrait for TypOpts {}

impl fmt::Display for TypOpts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            TypOpts::int => write!(f, "int"),
            TypOpts::flt => write!(f, "flt"),
            TypOpts::chr => write!(f, "chr"),
            TypOpts::bol => write!(f, "bol"),
            TypOpts::arr => write!(f, "arr"),
            TypOpts::vec => write!(f, "vec"),
            TypOpts::seq => write!(f, "seq"),
            TypOpts::mat => write!(f, "mat"),
            TypOpts::set => write!(f, "set"),
            TypOpts::map => write!(f, "map"),
            TypOpts::axi => write!(f, "axi"),
            TypOpts::tab => write!(f, "tab"),
            TypOpts::r#str => write!(f, "str"),
            TypOpts::num => write!(f, "num"),
            TypOpts::ptr => write!(f, "ptr"),
            TypOpts::err => write!(f, "err"),
            TypOpts::opt => write!(f, "opt"),
            TypOpts::nev => write!(f, "nev"),
            TypOpts::uni => write!(f, "uni"),
            TypOpts::any => write!(f, "any"),
            TypOpts::non => write!(f, "non"),
            TypOpts::nil => write!(f, "nil"),
            TypOpts::rec => write!(f, "rec"),
            TypOpts::ent => write!(f, "ent"),
            TypOpts::blu => write!(f, "blu"),
            TypOpts::std => write!(f, "std"),
            TypOpts::loc => write!(f, "loc"),
            TypOpts::url => write!(f, "url"),
            TypOpts::blk => write!(f, "blk"),
            TypOpts::rut => write!(f, "rut"),
            TypOpts::pat => write!(f, "pat"),
            TypOpts::gen => write!(f, "gen"),
        }
    }
}
