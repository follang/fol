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

use crate::error::flaw::Con;

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
            if let Err(e) = self.parse_stat(lex, flaw) { lex.to_endline(flaw); lex.eat_termin(flaw); };
        }
    }
    pub fn parse_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Con<()> {
        if lex.prev().key().is_terminal() || lex.prev().key().is_eof() {
            if matches!( lex.curr().key(), KEYWORD::assign(ASSIGN::var_) ) ||
                ( matches!( lex.curr().key(), KEYWORD::option(_) ) && matches!( lex.next().key(), KEYWORD::assign(ASSIGN::var_) ) ) {
                parse_stat_var(self, lex, flaw, &mut var_stat::init(), false)?;
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
            }
            lex.eat_termin(flaw);
        }
        Ok(())
    }
}


//------------------------------------------------------------------------------------------------------//
//                                             VAR STATEMENT
//------------------------------------------------------------------------------------------------------//
fn parse_stat_var(forest: &mut forest, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW, var_stat: &mut var_stat, recursive: bool) -> Con<()> {
    let loc = lex.curr().loc().clone();
    let mut opts: Vec<assign_opts> = Vec::new();
    let mut ids: Vec<Box<String>> = Vec::new();
    let mut typ: Vec<Box<stat>> = Vec::new();

    if !recursive {
        help_assign_definition(&mut opts, lex, flaw, var_stat, help_assign_var_options)?;
        if matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::roundO_)) {
            help_assign_recursive(forest, lex, flaw, var_stat, parse_stat_var)?;
            return Ok(())
        }
    }

    // identifier and indentifier list
    help_assign_identifiers(&mut ids, lex, flaw, var_stat)?;

    // types and types list
    help_assign_retypes(&mut typ, lex, flaw, var_stat)?;

    if ids.len() < typ.len() {
        lex.report_type_disbalance((" ".to_string() + ids.len().to_string().as_str() + " ").black().bold().on_white().to_string(),
        (" ".to_string() + typ.len().to_string().as_str() + " ").black().bold().on_white().to_string(),
            lex.curr().loc().clone(), flaw);
        return Err(flaw::flaw_type::parser(flaw::parser::parser_missmatch))
    }

    if matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::equal_)) || matches!(lex.look().key(), KEYWORD::operator(OPERATOR::assign2_)) {
        lex.eat_space(flaw);
        var_stat.set_body(parse_expr_var(lex, flaw));
    }

    // endline
    if lex.look().key().is_terminal() {
        if typ.len() == 0 && matches!(var_stat.get_body(), None) {
            lex.report_no_type(lex.past().key().to_string(), lex.past().loc().clone(), flaw);
            return Err(flaw::flaw_type::parser(flaw::parser::parser_no_type))
        }
        if typ.len() == 0 {
            var_stat.set_multi(None);
            var_stat.set_ident(ids[0].clone());
            forest.trees.push(tree::new(loc.clone(), body::stat(stat::Var(var_stat.clone()))));
        } else if typ.len() == ids.len() {
            for ((i, e), t) in ids.iter().enumerate().zip(typ.iter()) {
                let mut var_clone = var_stat.clone();
                var_clone.set_ident(e.clone());
                var_clone.set_retype(Some(t.clone()));
                if ids.len() > 1 { var_clone.set_multi(Some((i, *ids[0].clone()))) };
                forest.trees.push(tree::new(loc.clone(), body::stat(stat::Var(var_clone))));
            }
        } else {
            for i in 0..typ.len() {
                let mut var_clone = var_stat.clone();
                var_clone.set_ident(ids[i].clone());
                var_clone.set_retype(Some(typ[i].clone()));
                var_clone.set_multi(Some((i, *ids[0].clone())));
                forest.trees.push(tree::new(loc.clone(), body::stat(stat::Var(var_clone))));
            }
            for i in typ.len()..ids.len() {
                let mut var_clone = var_stat.clone();
                var_clone.set_ident(ids[i].clone());
                var_clone.set_retype(Some(typ[typ.len()-1].clone()));
                var_clone.set_multi(Some((i, *ids[0].clone())));
                forest.trees.push(tree::new(loc.clone(), body::stat(stat::Var(var_clone))));
            }
        }
        return Ok(());
    }


    let msg = KEYWORD::symbol(SYMBOL::colon_).to_string()
        + " or " + KEYWORD::symbol(SYMBOL::comma_).to_string().as_str()
        + " or " + KEYWORD::symbol(SYMBOL::semi_).to_string().as_str()
        + " or " + KEYWORD::symbol(SYMBOL::equal_).to_string().as_str()
        + " or " + KEYWORD::operator(OPERATOR::assign2_).to_string().as_str();
    lex.report_many_unexpected(msg, lex.look().loc().clone(), flaw);
    return Err(flaw::flaw_type::parser(flaw::parser::parser_unexpected))
}

fn parse_expr_var(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<body>> {
    lex.to_endline(flaw);
    Some(Box::new(body::stat(stat::Illegal)))
}

fn help_assign_var_options(v: &mut Vec<assign_opts>, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Con<()> {
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
                return Err(flaw::flaw_type::parser(flaw::parser::parser_unexpected))
            }
        };
        v.push(el);
        lex.bump();
        return Ok(())
    }
    let deep = lex.curr().loc().deep();
    lex.bump();
    loop {
        //TODO: finish options
        if ( matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::squarC_)) && lex.curr().loc().deep() < deep )
            || lex.curr().key().is_eof() { break }
        lex.bump();
    }
    lex.bump();
    Ok(())
}

//------------------------------------------------------------------------------------------------------//
//                                                 HELPERS                                              //
//------------------------------------------------------------------------------------------------------//
fn help_assign_recursive(forest: &mut forest, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW, var_stat: &mut var_stat,
    assign: fn(&mut forest, &mut lexer::BAG, &mut flaw::FLAW, &mut var_stat, bool) -> Con<()> ) -> Con<()> {
    if matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::roundO_)) {
        lex.bump(); lex.eat_space(flaw);
        while matches!(lex.curr().key(), KEYWORD::ident(_)) {
            assign(forest, lex, flaw, var_stat, true)?;
            lex.eat_termin(flaw);
        }
        if matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::roundC_)) {
            lex.bump(); lex.eat_termin(flaw);
        } else {
            lex.report_unepected(KEYWORD::symbol(SYMBOL::roundC_).to_string(), lex.curr().loc().clone(), flaw);
            return Err(flaw::flaw_type::parser(flaw::parser::parser_unexpected))
        }
    }
    Ok(())
}
fn help_assign_identifiers(list: &mut Vec<Box<String>>, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW, var_stat: &mut var_stat) -> Con<()> {
    //identifier
    if !lex.look().key().is_ident() {
        lex.report_unepected(KEYWORD::ident(None).to_string(), lex.curr().loc().clone(), flaw);
        return Err(flaw::flaw_type::parser(flaw::parser::parser_unexpected))
    }
    while lex.look().key().is_ident() {
        lex.eat_space(flaw);
        list.push(parse_ident_stat(lex, flaw));
        if !(matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::comma_))) {
            break;
        }
        lex.jump();
    }
    Ok(())
}

fn help_assign_retypes(types: &mut Vec<Box<stat>>, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW, var_stat: &mut var_stat) -> Con<()> {
    if matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::colon_)) {
        lex.jump();
        if matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::equal_)) { return Ok(()) }
        // types
        if !lex.look().key().is_type() {
            lex.report_unepected(KEYWORD::types(TYPE::ANY).to_string(), lex.curr().loc().clone(), flaw);
            return Err(flaw::flaw_type::parser(flaw::parser::parser_unexpected))
        }
        while lex.look().key().is_type() {
            lex.eat_space(flaw);
            types.push(parse_type_stat(lex, flaw));
            if !(matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::comma_))) {
                break;
            }
            lex.jump();
        }
    }
    Ok(())
}

fn help_assign_definition(opts: &mut Vec<assign_opts>, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW, var_stat: &mut var_stat,
    assign: fn(&mut Vec<assign_opts>, &mut lexer::BAG, &mut flaw::FLAW) -> Con<()> ) -> Con<()> {
        // option symbol
        if matches!(lex.curr().key(), KEYWORD::option(_)) {
            assign(opts, lex, flaw)?;
        }
        // eat the entry (var, fun, typ...)
        lex.bump();
        // option elements
        if matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::squarO_)) {
            // ERROR if space betwwen 'var' and '['
            if !(matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::squarO_))) {
                lex.report_space_rem(lex.curr().loc().clone(), flaw);
                return Err(flaw::flaw_type::parser(flaw::parser::parser_space_rem))
            }
            assign(opts, lex, flaw)?;
        }
        var_stat.set_options(opts.clone());

        // ERROR if not 'space'
        if !(matches!(lex.curr().key(), KEYWORD::void(VOID::space_))) {
            lex.report_space_add(lex.prev().key().to_string(), lex.prev().loc().clone(), flaw);
            return Err(flaw::flaw_type::parser(flaw::parser::parser_space_add))
        }
        lex.eat_space(flaw);
        Ok(())
}


//------------------------------------------------------------------------------------------------------//
//                                             TYPE STATEMENT                                           //
//------------------------------------------------------------------------------------------------------//
fn parse_ident_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<String> {
    let to_ret = Box::new(lex.curr().con().clone());
    lex.jump();
    to_ret
}
fn parse_type_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
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
        _ => { temp_go_end_type(lex); Box::new(stat::Illegal) }
    }
}
// int
fn retypes_int_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Int));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// flt
fn retypes_flt_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Flt));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// chr
fn retypes_chr_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Chr));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// bol
fn retypes_bol_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Bol));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// arr
fn retypes_arr_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Arr));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// vec
fn retypes_vec_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Vec));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// seq
fn retypes_seq_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Seq));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// mat
fn retypes_mat_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Mat));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// set
fn retypes_set_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Set));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// map
fn retypes_map_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Map));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// axi
fn retypes_axi_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Axi));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// tab
fn retypes_tab_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Tab));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// str
fn retypes_str_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Str));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// num
fn retypes_num_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Num));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// ptr
fn retypes_ptr_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Ptr));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// err
fn retypes_err_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Err));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// opt
fn retypes_opt_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Opt));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// nev
fn retypes_nev_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Nev));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// uni
fn retypes_uni_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Uni));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// any
fn retypes_any_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Any));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// non
fn retypes_non_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Non));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// nil
fn retypes_nil_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Nil));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// rec
fn retypes_rec_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Rec));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// ent
fn retypes_ent_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Ent));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// blu
fn retypes_blu_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Blu));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// std
fn retypes_std_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Std));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// loc
fn retypes_loc_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Loc));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// url
fn retypes_url_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Url));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// blk
fn retypes_blk_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Blk));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// rut
fn retypes_rut_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Rut));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// pat
fn retypes_pat_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Pat));
    lex.bump();
    temp_go_end_type(lex);
    typ
}
// gen
fn retypes_gen_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Box<stat> {
    let typ = Box::new(stat::Typ(type_expr::Gen));
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
