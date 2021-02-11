use std::fmt;
use crate::syntax::nodes::{NodeTrait, StatTrait, Nodes, Node};
use crate::syntax::token::{KEYWORD, TYPE};

#[derive(Clone)]
pub enum NodeExprTypeType {
    int_,
    flt_,
    chr_,
    bol_,
    arr_,
    vec_,
    seq_,
    mat_,
    set_,
    map_,
    axi_,
    tab_,
    str_,
    num_,
    ptr_,
    err_,
    opt_,
    nev_,
    uni_,
    any_,
    non_,
    nil_,
    rec_,
    ent_,
    blu_,
    std_,
    loc_,
    url_,
    blk_,
    rut_,
    pat_,
    gen_,
}

impl NodeTrait for NodeExprTypeType {}
impl StatTrait for NodeExprTypeType {}

impl fmt::Display for NodeExprTypeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            NodeExprTypeType::int_ => write!(f, "int"),
            NodeExprTypeType::flt_ => write!(f, "flt"),
            NodeExprTypeType::chr_ => write!(f, "chr"),
            NodeExprTypeType::bol_ => write!(f, "bol"),
            NodeExprTypeType::arr_ => write!(f, "arr"),
            NodeExprTypeType::vec_ => write!(f, "vec"),
            NodeExprTypeType::seq_ => write!(f, "seq"),
            NodeExprTypeType::mat_ => write!(f, "mat"),
            NodeExprTypeType::set_ => write!(f, "set"),
            NodeExprTypeType::map_ => write!(f, "map"),
            NodeExprTypeType::axi_ => write!(f, "axi"),
            NodeExprTypeType::tab_ => write!(f, "tab"),
            NodeExprTypeType::str_ => write!(f, "str"),
            NodeExprTypeType::num_ => write!(f, "num"),
            NodeExprTypeType::ptr_ => write!(f, "ptr"),
            NodeExprTypeType::err_ => write!(f, "err"),
            NodeExprTypeType::opt_ => write!(f, "opt"),
            NodeExprTypeType::nev_ => write!(f, "nev"),
            NodeExprTypeType::uni_ => write!(f, "uni"),
            NodeExprTypeType::any_ => write!(f, "any"),
            NodeExprTypeType::non_ => write!(f, "non"),
            NodeExprTypeType::nil_ => write!(f, "nil"),
            NodeExprTypeType::rec_ => write!(f, "rec"),
            NodeExprTypeType::ent_ => write!(f, "ent"),
            NodeExprTypeType::blu_ => write!(f, "blu"),
            NodeExprTypeType::std_ => write!(f, "std"),
            NodeExprTypeType::loc_ => write!(f, "loc"),
            NodeExprTypeType::url_ => write!(f, "url"),
            NodeExprTypeType::blk_ => write!(f, "blk"),
            NodeExprTypeType::rut_ => write!(f, "rut"),
            NodeExprTypeType::pat_ => write!(f, "pat"),
            NodeExprTypeType::gen_ => write!(f, "gen"),
        }
    }
}

#[derive(Clone)]
pub struct NodeExprDatatype {
    datatype: Node,
    options: Option<Nodes>,
    restrict: Option<Nodes>,
}

impl From<TYPE> for NodeExprDatatype {
    fn from(key: TYPE) -> Self {
        match key {
            TYPE::int_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::int_)), options: None, restrict: None},
            TYPE::flt_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::flt_)), options: None, restrict: None},
            TYPE::chr_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::chr_)), options: None, restrict: None},
            TYPE::bol_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::bol_)), options: None, restrict: None},
            TYPE::arr_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::arr_)), options: None, restrict: None},
            TYPE::vec_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::vec_)), options: None, restrict: None},
            TYPE::seq_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::seq_)), options: None, restrict: None},
            TYPE::mat_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::mat_)), options: None, restrict: None},
            TYPE::set_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::set_)), options: None, restrict: None},
            TYPE::map_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::map_)), options: None, restrict: None},
            TYPE::axi_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::axi_)), options: None, restrict: None},
            TYPE::tab_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::tab_)), options: None, restrict: None},
            TYPE::str_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::str_)), options: None, restrict: None},
            TYPE::num_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::num_)), options: None, restrict: None},
            TYPE::ptr_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::ptr_)), options: None, restrict: None},
            TYPE::err_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::err_)), options: None, restrict: None},
            TYPE::opt_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::opt_)), options: None, restrict: None},
            TYPE::nev_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::nev_)), options: None, restrict: None},
            TYPE::uni_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::uni_)), options: None, restrict: None},
            TYPE::any_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::any_)), options: None, restrict: None},
            TYPE::non_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::non_)), options: None, restrict: None},
            TYPE::nil_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::nil_)), options: None, restrict: None},
            TYPE::rec_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::rec_)), options: None, restrict: None},
            TYPE::ent_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::ent_)), options: None, restrict: None},
            TYPE::blu_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::blu_)), options: None, restrict: None},
            TYPE::std_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::std_)), options: None, restrict: None},
            TYPE::loc_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::loc_)), options: None, restrict: None},
            TYPE::url_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::url_)), options: None, restrict: None},
            TYPE::blk_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::blk_)), options: None, restrict: None},
            TYPE::rut_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::rut_)), options: None, restrict: None},
            TYPE::pat_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::pat_)), options: None, restrict: None},
            TYPE::gen_ => NodeExprDatatype{datatype: Node::new(Box::new(NodeExprTypeType::gen_)), options: None, restrict: None},
            _ => unreachable!()
        }
    }
}

impl NodeTrait for NodeExprDatatype {}
impl StatTrait for NodeExprDatatype {}

impl fmt::Display for NodeExprDatatype {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let opts = match self.options { Some(ref e) => e.to_string(), None => String::new()  };
        let rest = match self.options { Some(ref e) => e.to_string(), None => String::new()  };
        write!(f, "{}[{}][{}]", self.datatype.to_string(), opts, rest)
    }
}
