#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::fmt;
use crate::parsing::lexer;
use crate::parsing::ast::*;
use crate::parsing::parser::*;
use crate::parsing::stat::retype::*;

use crate::scanning::token::*;
use crate::scanning::locate;
use crate::error::flaw;
use colored::Colorize;
use crate::error::flaw::Con;

pub fn parse_ident_stat(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> ID<String> {
    let to_ret = ID::new(lex.curr().loc().clone(), lex.curr().con().clone());
    lex.jump();
    to_ret
}

pub fn assign_recursive(forest: &mut forest, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW, op: Option<trees>,
    assign: fn(&mut forest, &mut lexer::BAG, &mut flaw::FLAW, op: Option<trees>) -> Con<()> ) -> Con<()> {
    if matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::roundO_)) {
        lex.bump(); lex.eat_space(flaw);
        while matches!(lex.curr().key(), KEYWORD::ident(_)) {
            assign(forest, lex, flaw, op.clone())?;
            lex.eat_termin(flaw);
        }
        if matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::roundC_)) {
            lex.bump();
        } else {
            lex.report_unepected(KEYWORD::symbol(SYMBOL::roundC_).to_string(), lex.curr().loc().clone(), flaw);
            return Err(flaw::flaw_type::parser(flaw::parser::parser_unexpected))
        }
    }
    Ok(())
}
pub fn assign_identifiers(list: &mut Vec<ID<String>>, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW, multi: bool) -> Con<()> {
    //identifier
    if !lex.look().key().is_ident() {
        lex.report_unepected(KEYWORD::ident(None).to_string(), lex.curr().loc().clone(), flaw);
        return Err(flaw::flaw_type::parser(flaw::parser::parser_unexpected))
    }
    while lex.look().key().is_ident() {
        lex.eat_space(flaw);
        list.push(parse_ident_stat(lex, flaw));
        if !(matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::comma_))) || !multi {
            break;
        }
        lex.jump();
    }
    Ok(())
}

pub fn assign_retypes(types: &mut Vec<tree>, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW, multi: bool) -> Con<()> {
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
            if !(matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::comma_))) || !multi {
                break;
            }
            lex.jump();
        }
    }
    Ok(())
}

pub fn assign_definition(opts: &mut trees, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW,
    assign: fn(&mut trees, &mut lexer::BAG, &mut flaw::FLAW) -> Con<()> ) -> Con<()> {
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

        // ERROR if not 'space'
        if !(matches!(lex.curr().key(), KEYWORD::void(VOID::space_))) {
            lex.report_space_add(lex.prev().key().to_string(), lex.prev().loc().clone(), flaw);
            return Err(flaw::flaw_type::parser(flaw::parser::parser_space_add))
        }
        lex.eat_space(flaw);
        Ok(())
}

pub fn assign_options(v: &mut trees, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Con<()> {
    if matches!(lex.curr().key(), KEYWORD::option(_)) {
        let el;
        match lex.curr().key() {
            KEYWORD::option(OPTION::mut_) => { el = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Opts(assign_opts::Mut))) }
            KEYWORD::option(OPTION::sta_) => { el = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Opts(assign_opts::Sta))) }
            KEYWORD::option(OPTION::exp_) => { el = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Opts(assign_opts::Exp))) }
            KEYWORD::option(OPTION::hid_) => { el = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Opts(assign_opts::Hid))) }
            KEYWORD::option(OPTION::hep_) => { el = tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Opts(assign_opts::Hep))) }
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

pub fn assign_contract(v: &mut trees, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Con<()> {
    // TODO:
    // TEMPOrARY GO TO END OF CONTRACT
    if matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::roundO_)) {
        let deep = lex.curr().loc().deep();
        while !( ( matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::roundC_))
            && lex.curr().loc().deep() < deep )
            || lex.curr().key().is_eof() ) {
            lex.bump();
        }
        lex.bump();
    }
    Ok(())
}
pub fn assign_generics(v: &mut trees, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Con<()> {
    // TODO:
    // TEMPOrARY GO TO END OF ENDGENERICS
    if matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::squarO_)) {
        let deep = lex.curr().loc().deep();
        while !( ( matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::squarC_))
            && lex.curr().loc().deep() < deep )
            || lex.curr().key().is_eof() ) {
            lex.bump();
        }
        lex.bump();
    }
    Ok(())
}

pub fn error_assign_last(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Con<()> {
    let msg = KEYWORD::symbol(SYMBOL::colon_).to_string()
        + " or " + KEYWORD::symbol(SYMBOL::comma_).to_string().as_str()
        + " or " + KEYWORD::symbol(SYMBOL::semi_).to_string().as_str()
        + " or " + KEYWORD::symbol(SYMBOL::equal_).to_string().as_str()
        + " or " + KEYWORD::operator(OPERATOR::assign2_).to_string().as_str();
    lex.report_many_unexpected(msg, lex.look().loc().clone(), flaw);
    return Err(flaw::flaw_type::parser(flaw::parser::parser_unexpected))
}

pub fn error_disbalance(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW, ids: &Vec<ID<String>>, typ: &Vec<tree>) -> Con<()> {
    if ids.len() < typ.len() {
        lex.report_type_disbalance((" ".to_string() + ids.len().to_string().as_str() + " ").black().bold().on_white().to_string(),
        (" ".to_string() + typ.len().to_string().as_str() + " ").black().bold().on_white().to_string(),
            lex.curr().loc().clone(), flaw);
        return Err(flaw::flaw_type::parser(flaw::parser::parser_missmatch))
    }
    Ok(())
}
