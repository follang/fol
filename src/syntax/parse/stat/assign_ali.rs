#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::parsing::ast::*;
use crate::parsing::lexer;
use crate::parsing::parser::*;
use crate::parsing::stat::helper;
use crate::parsing::stat::retype::*;

use crate::error::flaw;
use crate::scanning::locate;
use crate::scanning::token::*;

use crate::error::flaw::Con;

pub fn parse_stat(
    forest: &mut forest,
    lex: &mut lexer::BAG,
    flaw: &mut flaw::FLAW,
    op: Option<trees>,
) -> Con<()> {
    //if let tree_type::stat(stat_type::Var(v)) = tree.get() {}
    let loc = lex.curr().loc().clone();
    let mut opt: trees;
    let mut ids: Vec<ID<String>> = Vec::new();
    let mut typ: Vec<tree> = Vec::new();
    let mut typ_stat = typ_stat::init();

    if let Some(options) = op.clone() {
        opt = options
    } else {
        opt = Vec::new();
        helper::assign_definition(&mut opt, lex, flaw, helper::assign_options)?;
        if matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::roundO_)) {
            lex.bump();
            lex.bump_termin(flaw);
            while matches!(lex.curr().key(), KEYWORD::ident(_)) {
                parse_stat(forest, lex, flaw, Some(opt.clone()))?;
                lex.bump_termin(flaw);
            }
            if matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::roundC_)) {
                lex.bump();
            } else {
                lex.report_unepected(
                    KEYWORD::symbol(SYMBOL::roundC_).to_string(),
                    lex.curr().loc().clone(),
                    flaw,
                );
                return Err(flaw::flaw_type::parser(flaw::parser::parser_unexpected));
            }
            // helper::assign_recursive(forest, lex, flaw, Some(opt), parse_stat_typ)?;
            lex.to_endline(flaw);
            lex.bump_termin(flaw);
            return Ok(());
        }
    }

    // identifier and indentifier list
    helper::assign_identifiers(&mut ids, lex, flaw, false)?;

    // generics
    helper::assign_generics(&mut opt, lex, flaw)?;

    // contracts
    helper::assign_contract(&mut opt, lex, flaw)?;

    // types and types list
    helper::assign_retypes(&mut typ, lex, flaw, false)?;

    // endline
    if lex.look().key().is_terminal() {
        typ_stat.set_options(Some(opt.clone()));
        if typ.len() == ids.len() {
            for ((i, e), t) in ids.iter().enumerate().zip(typ.iter()) {
                let mut var_clone = typ_stat.clone();
                var_clone.set_ident(tree::new(
                    lex.curr().loc().clone(),
                    tree_type::stat(stat_type::Ident(ids[i].get().to_string())),
                ));
                var_clone.set_retype(Some(t.clone()));
                if ids.len() > 1 {
                    var_clone.set_multi(Some((i, ids[0].get().to_string())))
                };
                forest.trees.push(tree::new(
                    lex.curr().loc().clone(),
                    tree_type::stat(stat_type::Ali(var_clone)),
                ));
            }
        } else {
            lex.report_type_disbalance(
                ids.len().to_string(),
                typ.len().to_string(),
                lex.prev().loc().clone(),
                flaw,
            );
            return Err(flaw::flaw_type::parser(flaw::parser::parser_missmatch));
        }
        lex.to_endline(flaw);
        lex.bump_termin(flaw);
        return Ok(());
    } else {
        lex.report_body_forbidden(lex.past().key().to_string(), lex.past().loc().clone(), flaw);
        return Err(flaw::flaw_type::parser(flaw::parser::parser_body_forbidden));
    }
}

fn parse_expr(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<tree> {
    lex.to_endline(flaw);
    Some(tree::new(
        lex.curr().loc().clone(),
        tree_type::stat(stat_type::Illegal),
    ))
}