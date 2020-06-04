#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::parsing::lexer;
use crate::parsing::ast::*;
use crate::parsing::stat::helper::*;

use crate::scanning::token::*;
use crate::scanning::locate;
use crate::error::flaw;
use colored::Colorize;

use crate::error::flaw::Con;

pub fn parse_type_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    match lex.curr().key() {
        KEYWORD::types(TYPE::int_) => { return retypes_int_stat(lex, flaw) }
        KEYWORD::types(TYPE::flt_) => { return retypes_flt_stat(lex, flaw) }
        KEYWORD::types(TYPE::chr_) => { return retypes_chr_stat(lex, flaw) }
        KEYWORD::types(TYPE::bol_) => { return retypes_bol_stat(lex, flaw) }
        KEYWORD::types(TYPE::arr_) => { return retypes_arr_stat(lex, flaw) }
        KEYWORD::types(TYPE::vec_) => { return retypes_vec_stat(lex, flaw) }
        KEYWORD::types(TYPE::seq_) => { return retypes_seq_stat(lex, flaw) }
        KEYWORD::types(TYPE::mat_) => { return retypes_mat_stat(lex, flaw) }
        KEYWORD::types(TYPE::set_) => { return retypes_set_stat(lex, flaw) }
        KEYWORD::types(TYPE::map_) => { return retypes_map_stat(lex, flaw) }
        KEYWORD::types(TYPE::axi_) => { return retypes_axi_stat(lex, flaw) }
        KEYWORD::types(TYPE::tab_) => { return retypes_tab_stat(lex, flaw) }
        KEYWORD::types(TYPE::str_) => { return retypes_str_stat(lex, flaw) }
        KEYWORD::types(TYPE::num_) => { return retypes_num_stat(lex, flaw) }
        KEYWORD::types(TYPE::ptr_) => { return retypes_ptr_stat(lex, flaw) }
        KEYWORD::types(TYPE::err_) => { return retypes_err_stat(lex, flaw) }
        KEYWORD::types(TYPE::opt_) => { return retypes_opt_stat(lex, flaw) }
        KEYWORD::types(TYPE::nev_) => { return retypes_nev_stat(lex, flaw) }
        KEYWORD::types(TYPE::uni_) => { return retypes_uni_stat(lex, flaw) }
        KEYWORD::types(TYPE::any_) => { return retypes_any_stat(lex, flaw) }
        KEYWORD::types(TYPE::non_) => { return retypes_non_stat(lex, flaw) }
        KEYWORD::types(TYPE::nil_) => { return retypes_nil_stat(lex, flaw) }
        KEYWORD::types(TYPE::rec_) => { return retypes_rec_stat(lex, flaw) }
        KEYWORD::types(TYPE::ent_) => { return retypes_ent_stat(lex, flaw) }
        KEYWORD::types(TYPE::blu_) => { return retypes_blu_stat(lex, flaw) }
        KEYWORD::types(TYPE::std_) => { return retypes_std_stat(lex, flaw) }
        KEYWORD::types(TYPE::loc_) => { return retypes_loc_stat(lex, flaw) }
        KEYWORD::types(TYPE::url_) => { return retypes_url_stat(lex, flaw) }
        KEYWORD::types(TYPE::blk_) => { return retypes_blk_stat(lex, flaw) }
        KEYWORD::types(TYPE::rut_) => { return retypes_rut_stat(lex, flaw) }
        KEYWORD::types(TYPE::pat_) => { return retypes_pat_stat(lex, flaw) }
        KEYWORD::types(TYPE::gen_) => { return retypes_gen_stat(lex, flaw) }
        _ => { temp_go_end_type(lex); tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Illegal)) }
    }
}
// int
fn retypes_int_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Int)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// flt
fn retypes_flt_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Flt)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// chr
fn retypes_chr_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Chr)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// bol
fn retypes_bol_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Bol)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// arr
fn retypes_arr_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Arr)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// vec
fn retypes_vec_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Vec)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// seq
fn retypes_seq_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Seq)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// mat
fn retypes_mat_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Mat)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// set
fn retypes_set_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Set)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// map
fn retypes_map_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Map)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// axi
fn retypes_axi_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Axi)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// tab
fn retypes_tab_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Tab)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// str
fn retypes_str_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Str)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// num
fn retypes_num_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Num)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// ptr
fn retypes_ptr_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Ptr)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// err
fn retypes_err_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Err)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// opt
fn retypes_opt_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Opt)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// nev
fn retypes_nev_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Nev)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// uni
fn retypes_uni_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Uni)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// any
fn retypes_any_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Any)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// non
fn retypes_non_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Non)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// nil
fn retypes_nil_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Nil)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// rec
fn retypes_rec_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Rec)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// ent
fn retypes_ent_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Ent)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// blu
fn retypes_blu_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Blu)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// std
fn retypes_std_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Std)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// loc
fn retypes_loc_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Loc)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// url
fn retypes_url_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Url)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// blk
fn retypes_blk_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Blk)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// rut
fn retypes_rut_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Rut)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// pat
fn retypes_pat_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Pat)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// gen
fn retypes_gen_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> tree {
    let typ = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Retype(retype_stat::Gen)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}

/// TEMPOrARY GO TO END VAR
fn temp_go_end_type(lex: &mut lexer::BAG) {
    if matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::squarO_)) {
        let deep = lex.curr().loc().deep();
        while !( ( matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::squarC_))
            && lex.curr().loc().deep() < deep )
            || lex.curr().key().is_eof() ) {
            lex.bump();
        }
        lex.bump();
    }
}
