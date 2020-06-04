#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::parsing::lexer;
use crate::parsing::ast::*;
use crate::parsing::parser::*;
use crate::parsing::stat::helper;
use crate::parsing::stat::retype::*;

use crate::scanning::token::*;
use crate::scanning::locate;
use crate::error::flaw;

use crate::error::flaw::Con;

pub fn parse_stat_typ(forest: &mut forest, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW, op: Option<Vec<assign_opts>>) -> Con<()> {
    //if let tree_type::stat(stat_type::Var(v)) = tree.get() {}
    let loc = lex.curr().loc().clone();
    let mut opt: Vec<assign_opts>;
    let mut ids: Vec<ID<String>> = Vec::new();
    let mut typ: Vec<tree> = Vec::new();
    let mut typ_stat = typ_stat::init();

    if let Some(o) = op.clone() {
        opt = o
    } else {
        opt = Vec::new();
        helper::assign_definition(&mut opt, lex, flaw, helper::assign_options)?;
        if matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::roundO_)) {
            helper::assign_recursive(forest, lex, flaw, Some(opt), parse_stat_typ)?;
            lex.to_endline(flaw); lex.eat_termin(flaw);
            return Ok(())
        }
    }

    // identifier and indentifier list
    helper::assign_identifiers(&mut ids, lex, flaw, false)?;
    lex.log(">>");

    // types and types list
    helper::assign_retypes(&mut typ, lex, flaw, false)?;

    if matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::equal_)) || matches!(lex.look().key(), KEYWORD::operator(OPERATOR::assign2_)) {
        lex.eat_space(flaw);
        typ_stat.set_body(parse_expr_typ(lex, flaw));
    }

    // endline
    if lex.look().key().is_terminal() {
        if typ.len() == 0 && matches!(typ_stat.get_body(), None) {
            lex.report_no_type(lex.past().key().to_string(), lex.past().loc().clone(), flaw);
            return Err(flaw::flaw_type::parser(flaw::parser::parser_no_type))
        }
        typ_stat.set_options(opt.clone());
        if typ.len() == 0 {
            typ_stat.set_multi(None);
            typ_stat.set_ident(tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Ident(ids[0].get().to_string()))));
            forest.trees.push(tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Typ(typ_stat.clone()))));
        } else {
            for ((i, e), t) in ids.iter().enumerate().zip(typ.iter()) {
                let mut var_clone = typ_stat.clone();
                var_clone.set_ident(tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Ident(ids[i].get().to_string()))));
                var_clone.set_retype(Some(t.clone()));
                if ids.len() > 1 { var_clone.set_multi(Some((i, ids[0].get().to_string()))) };
                forest.trees.push(tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Typ(var_clone))));
            }
        }
        lex.to_endline(flaw); lex.eat_termin(flaw);
        return Ok(());
    }
    return helper::error_assign_last(lex, flaw);
}

fn parse_expr_typ(lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<tree> {
    lex.to_endline(flaw);
    Some(tree::new(lex.curr().loc().clone(), tree_type::stat(stat_type::Illegal)))
}
