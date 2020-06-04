#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::parsing::lexer;
use crate::parsing::ast::*;
use crate::parsing::parser::*;
use crate::parsing::stat::helper::*;
use crate::parsing::stat::retype::*;

use crate::scanning::token::*;
use crate::scanning::locate;
use crate::error::flaw;
use colored::Colorize;

use crate::error::flaw::Con;


pub fn parse_stat_var(forest: &mut forest, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW, op: Option<Vec<assign_opts>>) -> Con<()> {
    //if let tree_type::stat(stat_type::Var(v)) = tree.get() {}
    let loc = lex.curr().loc().clone();
    let mut opt: Vec<assign_opts>;
    let mut ids: Vec<ID<String>> = Vec::new();
    let mut typ: Vec<tree> = Vec::new();
    let mut var_stat = var_stat::init();

    // go recursively if None or go normal if Some
    if let Some(optioins) = op.clone() {
        opt = optioins
    } else {
        opt = Vec::new();
        help_assign_definition(&mut opt, lex, flaw, help_assign_options)?;
        if matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::roundO_)) {
            help_assign_recursive(forest, lex, flaw, Some(opt), parse_stat_var)?;
            lex.to_endline(flaw); lex.eat_termin(flaw);
            return Ok(())
        }
    }

    // identifier and indentifier list
    help_assign_identifiers(&mut ids, lex, flaw, true)?;

    // types and types list
    help_assign_retypes(&mut typ, lex, flaw, true)?;

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
        var_stat.set_options(opt.clone());
        if typ.len() == 0 {
            var_stat.set_multi(None);
            var_stat.set_ident(tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Ident(ids[0].get().to_string()))));
            forest.trees.push(tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Var(var_stat.clone()))));
        } else if typ.len() == ids.len() {
            for ((i, e), t) in ids.iter().enumerate().zip(typ.iter()) {
                let mut var_clone = var_stat.clone();
                var_clone.set_ident(tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Ident(ids[i].get().to_string()))));
                var_clone.set_retype(Some(t.clone()));
                if ids.len() > 1 { var_clone.set_multi(Some((i, ids[0].get().to_string()))) };
                forest.trees.push(tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Var(var_clone))));
            }
        } else {
            for i in 0..typ.len() {
                let mut var_clone = var_stat.clone();
                var_clone.set_ident(tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Ident(ids[i].get().to_string()))));
                var_clone.set_retype(Some(typ[i].clone()));
                var_clone.set_multi(Some((i, ids[0].get().to_string())));
                forest.trees.push(tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Var(var_clone))));
            }
            for i in typ.len()..ids.len() {
                let mut var_clone = var_stat.clone();
                var_clone.set_ident(tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Ident(ids[i].get().to_string()))));
                var_clone.set_retype(Some(typ[typ.len()-1].clone()));
                var_clone.set_multi(Some((i, ids[0].get().to_string())));
                forest.trees.push(tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Var(var_clone))));
            }
        }
        lex.to_endline(flaw); lex.eat_termin(flaw);
        return Ok(());
    }

    return error_assign_last(lex, flaw);
}

fn parse_expr_var(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<tree> {
    lex.to_endline(flaw);
    Some(tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Illegal)))
}