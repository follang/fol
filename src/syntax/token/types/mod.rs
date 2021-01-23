use colored::Colorize;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TYPE {
    ANY,
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

impl fmt::Display for TYPE {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t;
        match self {
            TYPE::int_ => { t = Some("int".to_string()); },
            TYPE::flt_ => { t = Some("flt".to_string()); },
            TYPE::chr_ => { t = Some("chr".to_string()); },
            TYPE::bol_ => { t = Some("bol".to_string()); },
            TYPE::arr_ => { t = Some("arr".to_string()); },
            TYPE::vec_ => { t = Some("vec".to_string()); },
            TYPE::seq_ => { t = Some("seq".to_string()); },
            TYPE::mat_ => { t = Some("mat".to_string()); },
            TYPE::set_ => { t = Some("set".to_string()); },
            TYPE::map_ => { t = Some("map".to_string()); },
            TYPE::axi_ => { t = Some("axi".to_string()); },
            TYPE::tab_ => { t = Some("tab".to_string()); },
            TYPE::str_ => { t = Some("str".to_string()); },
            TYPE::num_ => { t = Some("num".to_string()); },
            TYPE::ptr_ => { t = Some("ptr".to_string()); },
            TYPE::err_ => { t = Some("err".to_string()); },
            TYPE::opt_ => { t = Some("opt".to_string()); },
            TYPE::nev_ => { t = Some("nev".to_string()); },
            TYPE::uni_ => { t = Some("uni".to_string()); },
            TYPE::any_ => { t = Some("any".to_string()); },
            TYPE::non_ => { t = Some("non".to_string()); },
            TYPE::nil_ => { t = Some("nil".to_string()); },
            TYPE::rec_ => { t = Some("rec".to_string()); },
            TYPE::ent_ => { t = Some("ent".to_string()); },
            TYPE::blu_ => { t = Some("blu".to_string()); },
            TYPE::std_ => { t = Some("std".to_string()); },
            TYPE::loc_ => { t = Some("loc".to_string()); },
            TYPE::url_ => { t = Some("url".to_string()); },
            TYPE::blk_ => { t = Some("blk".to_string()); },
            TYPE::rut_ => { t = Some("rut".to_string()); },
            TYPE::pat_ => { t = Some("pat".to_string()); },
            TYPE::gen_ => { t = Some("gen".to_string()); },
            _ => { t = None },
        };
        write!(f, "{}  {}",
            " TYPE     ".black().on_red(),
            match t { 
                Some(val) => { (format!(" {} ", val)).black().on_red().to_string() }, 
                None => "".to_string()
            },
        )
    }
}
