#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_macros)]

use std::fmt;
use crate::parsing::lexer;
use crate::parsing::ast::*;
use crate::scanning::token::*;
use crate::scanning::locate;
use crate::error::flaw;
use colored::Colorize;


pub struct forest {
    pub trees: Vec<tree>
}

pub fn new() -> forest {
    forest{ trees: Vec::new() }
}

impl forest {
    pub fn init(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) {
        if !flaw.list().is_empty() { return; }
        let f = self::new();
        while lex.not_empty() {
            self.parse_stat(lex, flaw);
        }
    }
    pub fn parse_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) {
        if lex.prev().key().is_terminal() || lex.prev().key().is_eof() {
            if matches!( lex.curr().key(), KEYWORD::assign(ASSIGN::var_) ) ||
                ( matches!( lex.curr().key(), KEYWORD::option(_) ) && matches!( lex.next().key(), KEYWORD::assign(ASSIGN::var_) ) ) {
                parse_stat_var(self, lex, flaw, &mut var_stat::init(), false);
            // } else if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::fun_) ) ||
                // ( matches!( l.curr().key(), KEYWORD::symbol(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::fun_) ) ) {
                // self.parse_stat_var(l, flaw);
            // } else if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::pro_) ) ||
                // ( matches!( l.curr().key(), KEYWORD::symbol(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::pro_) ) ) {
                // self.parse_stat_var(l, flaw);
            // } else if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::log_) ) ||
                // ( matches!( l.curr().key(), KEYWORD::symbol(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::log_) ) ) {
                // self.parse_stat_var(l, flaw);
            // } else if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::typ_) ) ||
                // ( matches!( l.curr().key(), KEYWORD::symbol(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::typ_) ) ) {
                // self.parse_stat_var(l, flaw);
            // } else if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::ali_) ) ||
                // ( matches!( l.curr().key(), KEYWORD::symbol(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::ali_) ) ) {
                // self.parse_stat_var(l, flaw);
            // } else if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::use_) ) ||
                // ( matches!( l.curr().key(), KEYWORD::symbol(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::use_) ) ) {
                // self.parse_stat_var(l, flaw);
            // } else if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::def_) ) ||
                // ( matches!( l.curr().key(), KEYWORD::symbol(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::def_) ) ) {
                // self.parse_stat_var(l, flaw);
            } else {
                lex.to_endline(flaw);
                lex.eat_termin(flaw);
            }
        }
    }
}


//------------------------------------------------------------------------------------------------------//
//                                             VAR STATEMENT                                            //
//------------------------------------------------------------------------------------------------------//
fn parse_stat_var(forest: &mut forest, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW, var_stat: &mut var_stat, group: bool) {
    let loc = lex.curr().loc().clone();
    let mut opts: Vec<assign_opts> = Vec::new();
    let identifier: String;
    let mut list: Vec<String> = Vec::new();
    let mut types: Vec<Option<Box<stat>>> = Vec::new();

    if !group {
        if !help_assign_definition(&mut opts, lex, flaw, var_stat, help_assign_var_options) { return; };
        if matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::roundO_)) {
            lex.bump(); lex.eat_space(flaw);
            while matches!(lex.curr().key(), KEYWORD::ident(_)) {
                parse_stat_var(forest, lex, flaw, var_stat, true);
                lex.eat_termin(flaw);
            }
            if matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::roundC_)) {
                lex.bump(); lex.eat_termin(flaw);
            } else {
                lex.report_unepected(KEYWORD::symbol(SYMBOL::roundC_).to_string(), lex.curr().loc().clone(), flaw);
                return
            }
            return
        }
    }


    // identifier and indentifier list
    identifier = lex.curr().con().clone();
    if !help_assign_identifiers(&mut list, lex, flaw, var_stat) { return; } ;

    // type separator ':'
    // identifier and indentifier list
    if !help_assign_retypes(&mut types, lex, flaw, var_stat) { return; };

    // error if list variables and list type does not match
    if list.len() != types.len() && types.len() != 0 {
        lex.report_type_disbalance((" ".to_string() + list.len().to_string().as_str() + " ").black().bold().on_white().to_string(),
        (" ".to_string() + types.len().to_string().as_str() + " ").black().bold().on_white().to_string(),
        lex.curr().loc().clone(), flaw);
        return
    }

    // if equal or endline
    if lex.look().key().is_terminal()
        || matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::equal_))
        || matches!(lex.look().key(), KEYWORD::operator(OPERATOR::assign2_))
    {
        lex.eat_space(flaw);
        if !lex.look().key().is_terminal(){
            var_stat.set_body(parse_expr_var(lex, flaw));
        }
        if list.len() == 0 {
            var_stat.set_multi(None);
            forest.trees.push(tree::new(loc.clone(), body::stat(stat::Var(var_stat.clone()))));
        } else {
            var_stat.set_multi(Some((0, identifier.clone())));
            forest.trees.push(tree::new(loc.clone(), body::stat(stat::Var(var_stat.clone()))));
            if types.len() != 0 {
                for ((i, e), f) in list.iter().enumerate().zip(types.iter()) {
                    let mut clo = var_stat.clone();
                    clo.set_multi(Some((i+1, identifier.clone())));
                    clo.set_ident(Box::new(e.clone()));
                    clo.set_retype(f.clone());
                    forest.trees.push(tree::new(loc.clone(), body::stat(stat::Var(clo))));
                }
            } else {
                for (i, e) in list.iter().enumerate() {
                    let mut clo = var_stat.clone();
                    clo.set_ident(Box::new(e.clone()));
                    clo.set_multi(Some((i+1, identifier.clone())));
                    forest.trees.push(tree::new(loc.clone(), body::stat(stat::Var(clo))));
                }
            }
        }
        lex.eat_termin(flaw);
        return;
    }

    let msg = KEYWORD::symbol(SYMBOL::colon_).to_string()
        + " or " + KEYWORD::symbol(SYMBOL::comma_).to_string().as_str()
        + " or " + KEYWORD::symbol(SYMBOL::semi_).to_string().as_str()
        + " or " + KEYWORD::symbol(SYMBOL::equal_).to_string().as_str()
        + " or " + KEYWORD::operator(OPERATOR::assign2_).to_string().as_str();
    lex.report_many_unexpected(msg, lex.curr().loc().clone(), flaw);
    return
}

fn parse_expr_var(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<body>> {
    lex.to_endline(flaw);
    lex.eat_termin(flaw);
    None
}

fn help_assign_var_options(v: &mut Vec<assign_opts>, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> bool {
    if matches!(lex.curr().key(), KEYWORD::option(_)) {
        let el;
        match lex.curr().key() {
            KEYWORD::option(OPTION::mut_) => { el = assign_opts::Mut }
            KEYWORD::option(OPTION::sta_) => { el = assign_opts::Sta }
            KEYWORD::option(OPTION::exp_) => { el = assign_opts::Exp }
            KEYWORD::option(OPTION::hid_) => { el = assign_opts::Hid }
            KEYWORD::option(OPTION::hep_) => { el = assign_opts::Hep }
            _ => {
                lex.report_unepected(KEYWORD::option(OPTION::ANY).to_string(), lex.curr().loc().clone(), flaw);
                return false
            }
        };
        v.push(el);
        lex.bump();
        return true
    }
    let deep = lex.curr().loc().deep() -1;
    lex.bump();
    loop {
        //TODO: finish options
        if ( matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::squarC_)) && lex.curr().loc().deep() == deep )
            || lex.curr().key().is_eof() { break }
        lex.bump();
    }
    lex.bump();
    return true
}

//------------------------------------------------------------------------------------------------------//
//                                                 HELPERS                                              //
//------------------------------------------------------------------------------------------------------//
fn help_assign_identifiers(list: &mut Vec<String>, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW, var_stat: &mut var_stat) -> bool {
    //identifier
    if matches!(lex.look().key(), KEYWORD::ident(_)) {
        lex.eat_space(flaw);
        var_stat.set_ident(Box::new(lex.curr().con().clone()));
        lex.bump();
    } else {
        lex.report_unepected(KEYWORD::ident(None).to_string(), lex.curr().loc().clone(), flaw);
        return false
    }
    // list variables
    while matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::comma_)) {
        lex.jump();
        if matches!(lex.look().key(), KEYWORD::ident(_)) {
            lex.eat_space(flaw);
            list.push(lex.curr().con().clone());
            lex.jump();
        } else {
            lex.report_unepected(KEYWORD::ident(None).to_string(), lex.curr().loc().clone(), flaw);
            return false
        }
    }
    return true
}

fn help_assign_retypes(types: &mut Vec<Option<Box<stat>>>, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW, var_stat: &mut var_stat) -> bool {
    if matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::colon_)) {
        lex.eat_space(flaw);
        lex.bump();
        lex.eat_space(flaw);
        if matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::equal_)) { return true }

        // types
        if matches!(lex.curr().key(), KEYWORD::types(_)) {
            var_stat.set_retype(parse_type_stat(lex, flaw));
        } else {
            lex.report_unepected(KEYWORD::types(TYPE::ANY).to_string(), lex.curr().loc().clone(), flaw);
            return false;
        }

        while matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::comma_)) {
            lex.jump();
            if matches!(lex.look().key(), KEYWORD::types(_)) {
                lex.eat_space(flaw);
                types.push(parse_type_stat(lex, flaw));
            } else {
                lex.report_unepected(KEYWORD::types(TYPE::ANY).to_string(), lex.curr().loc().clone(), flaw);
                return false
            }
        }
    }
    return true
}

fn help_assign_definition(opts: &mut Vec<assign_opts>, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW, var_stat: &mut var_stat,
    assign: fn(&mut Vec<assign_opts>, &mut lexer::BAG, &mut flaw::FLAW) -> bool
    ) -> bool {
        // option symbol
        if matches!(lex.curr().key(), KEYWORD::option(_)) {
            assign(opts, lex, flaw);
        }
        // eat the entry (var, fun, typ...)
        lex.bump();
        // option elements
        if matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::squarO_)) {
            // ERROR if space betwwen 'var' and '['
            if !(matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::squarO_))) {
                lex.report_space_rem(lex.curr().loc().clone(), flaw);
                return false;
            }
            assign(opts, lex, flaw);
        }
        var_stat.set_options(opts.clone());

        // ERROR if not 'space'
        if !(matches!(lex.curr().key(), KEYWORD::void(VOID::space_))) {
            lex.report_space_add(lex.prev().key().to_string(), lex.prev().loc().clone(), flaw);
            return false;
        }
        lex.eat_space(flaw);
        return true
}


//------------------------------------------------------------------------------------------------------//
//                                             TYPE STATEMENT                                           //
//------------------------------------------------------------------------------------------------------//
fn parse_type_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
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
        _ => { return retypes_all_stat(lex, flaw) }
    }
}
// int
fn retypes_int_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Int)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// flt
fn retypes_flt_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Flt)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// chr
fn retypes_chr_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Chr)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// bol
fn retypes_bol_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Bol)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// arr
fn retypes_arr_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Arr)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// vec
fn retypes_vec_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Vec)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// seq
fn retypes_seq_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Seq)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// mat
fn retypes_mat_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Mat)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// set
fn retypes_set_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Set)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// map
fn retypes_map_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Map)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// axi
fn retypes_axi_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Axi)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// tab
fn retypes_tab_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Tab)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// str
fn retypes_str_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Str)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// num
fn retypes_num_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Num)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// ptr
fn retypes_ptr_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Ptr)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// err
fn retypes_err_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Err)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// opt
fn retypes_opt_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Opt)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// nev
fn retypes_nev_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Nev)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// uni
fn retypes_uni_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Uni)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// any
fn retypes_any_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Any)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// non
fn retypes_non_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Non)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// nil
fn retypes_nil_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Nil)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// rec
fn retypes_rec_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Rec)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// ent
fn retypes_ent_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Ent)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// blu
fn retypes_blu_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Blu)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// std
fn retypes_std_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Std)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// loc
fn retypes_loc_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Loc)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// url
fn retypes_url_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Url)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// blk
fn retypes_blk_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Blk)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// rut
fn retypes_rut_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Rut)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// pat
fn retypes_pat_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Pat)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// gen
fn retypes_gen_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    let typ = Some(Box::new(stat::Type(type_expr::Gen)));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// ANY
fn retypes_all_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
    lex.bump();
    temp_go_end_type(lex);
    None
}
/// TEMPOrARY GO TO END VAR
fn temp_go_end_type(lex: &mut lexer::BAG) {
    if matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::squarO_)) {
        let deep = lex.curr().loc().deep() -1;
        while !( ( matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::squarC_)) && lex.curr().loc().deep() == deep ) || lex.curr().key().is_eof() ) {
            lex.bump();
        }
        lex.bump();
    }
}