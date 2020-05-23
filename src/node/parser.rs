#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_macros)]

use std::fmt;
use crate::node::lexer;
use crate::node::ast::*;
use crate::scan::token::*;
use crate::scan::locate;
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
                self.parse_stat_var(lex, flaw, &mut var_stat::init(), false);
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
impl forest {
    pub fn parse_stat_var(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW, mut var_stat: &mut var_stat, group: bool) {
        let loc = lex.curr().loc().clone();
        let mut options: Vec<assign_opts> = Vec::new();
        let identifier: String;
        let mut list: Vec<String> = Vec::new();
        let mut types: Vec<Option<Box<stat>>> = Vec::new();

        if !group {
            // option symbol
            if matches!(lex.curr().key(), KEYWORD::option(_)) {
                self.help_assign_options(&mut options, lex, flaw);
            }

            // eat assign var
            lex.bump();

            // option elements
            if matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::squarO_)) {
                // ERROR if space betwwen 'var' and '['
                if !(matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::squarO_))) {
                    lex.report_space_rem(lex.curr().loc().clone(), flaw);
                    return;
                }
                self.help_assign_options(&mut options, lex, flaw);
            }
            var_stat.set_options(options);

            // ERROR if not 'space'
            if !(matches!(lex.curr().key(), KEYWORD::void(VOID::space_))) {
                lex.report_space_add(lex.prev().key().to_string(), lex.prev().loc().clone(), flaw); return;
            } else { lex.eat_space(flaw); }


            // group variables matching "("
            if matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::roundO_)) {
                lex.bump(); lex.eat_space(flaw);
                while matches!(lex.curr().key(), KEYWORD::ident(_)) {
                    self.parse_stat_var(lex, flaw, &mut var_stat, true);
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

        //identifier
        if matches!(lex.curr().key(), KEYWORD::ident(_)) {
            identifier = lex.curr().con().clone();
            var_stat.set_ident(Box::new(identifier.clone()));
            lex.bump();
        } else {
            lex.report_unepected(KEYWORD::ident(None).to_string(), lex.curr().loc().clone(), flaw);
            return
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
                return
            }
        }


        // type separator ':'
        if matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::colon_)) {
            if !(matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::colon_))) {
                lex.report_space_rem(lex.curr().loc().clone(), flaw);
                return
            }
            lex.bump();

            // ERROR if not 'space'
            if !(matches!(lex.curr().key(), KEYWORD::void(VOID::space_))) {
                lex.report_space_add(lex.prev().key().to_string(), lex.prev().loc().clone(), flaw);
                return;
            } else { lex.eat_space(flaw); }

            // ERROR if not any 'type'
            if !(matches!(lex.curr().key(), KEYWORD::types(_))) {
                lex.report_unepected(KEYWORD::types(TYPE::ANY).to_string(), lex.curr().loc().clone(), flaw);
                return;

            // types
            } else {
                var_stat.set_retype(self.parse_type_stat(lex, flaw));
            }
        }

        // list types
        while matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::comma_)) {
            lex.jump();
            if matches!(lex.look().key(), KEYWORD::types(_)) {
                lex.eat_space(flaw);
                types.push(self.parse_type_stat(lex, flaw));
            } else {
                lex.report_unepected(KEYWORD::types(TYPE::ANY).to_string(), lex.curr().loc().clone(), flaw);
                return
            }
        }
        // error if list variables and list type does not match
        if list.len() != types.len() && types.len() != 0 {
            lex.report_type_disbalance((" ".to_string() + list.len().to_string().as_str() + " ").black().bold().on_white().to_string(),
            (" ".to_string() + types.len().to_string().as_str() + " ").black().bold().on_white().to_string(),
            lex.curr().loc().clone(), flaw);
            // return
        }

        // if equal or endline
        if lex.look().key().is_terminal() || matches!(lex.look().key(), KEYWORD::operator(OPERATOR::assign_)) {
            lex.eat_space(flaw);
            if matches!(lex.look().key(), KEYWORD::operator(OPERATOR::assign_)){
                var_stat.set_body(self.parse_expr_var(lex, flaw));
            }
            if list.len() == 0 {
                var_stat.set_multi(None);
                self.trees.push(tree::new(loc.clone(), body::stat(stat::Var(var_stat.clone()))));
            } else {
                var_stat.set_multi(Some((0, identifier.clone())));
                self.trees.push(tree::new(loc.clone(), body::stat(stat::Var(var_stat.clone()))));
                if types.len() != 0 {
                    for ((i, e), f) in list.iter().enumerate().zip(types.iter()) {
                        let mut clo = var_stat.clone();
                        clo.set_multi(Some((i+1, identifier.clone())));
                        clo.set_ident(Box::new(e.clone()));
                        clo.set_retype(f.clone());
                        self.trees.push(tree::new(loc.clone(), body::stat(stat::Var(clo))));
                    }
                } else {
                    for (i, e) in list.iter().enumerate() {
                        let mut clo = var_stat.clone();
                        clo.set_ident(Box::new(e.clone()));
                        clo.set_multi(Some((i+1, identifier.clone())));
                        self.trees.push(tree::new(loc.clone(), body::stat(stat::Var(clo))));
                    }
                }
            }
            lex.eat_termin(flaw);
            return;
        }

        if matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::equal_)) {
            lex.report_space_add(lex.prev().key().to_string(), lex.prev().loc().clone(), flaw);
            return;
        }

        let msg = KEYWORD::symbol(SYMBOL::colon_).to_string()
            + " or " + KEYWORD::symbol(SYMBOL::comma_).to_string().as_str()
            + " or " + KEYWORD::symbol(SYMBOL::semi_).to_string().as_str()
            + " or " + KEYWORD::operator(OPERATOR::assign_).to_string().as_str();
        lex.report_many_unexpected(msg, lex.curr().loc().clone(), flaw);
        return
    }

    pub fn help_var_multipe_assign(&mut self, list: &mut Vec<String>, types: &mut Vec<Option<Box<stat>>>, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW, ) {

    }

    pub fn help_assign_options(&mut self, v: &mut Vec<assign_opts>, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) {
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
                    return
                }
            };
            v.push(el);
            lex.bump();
            return
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
    }
    pub fn parse_expr_var(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<body>> {
        lex.to_endline(flaw);
        lex.eat_termin(flaw);
        None
    }
}


//------------------------------------------------------------------------------------------------------//
//                                             TYPE STATEMENT                                           //
//------------------------------------------------------------------------------------------------------//
impl forest {
    pub fn parse_type_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        match lex.curr().key() {
            KEYWORD::types(TYPE::int_) => { return self.retypes_int_stat(lex, flaw) }
            KEYWORD::types(TYPE::flt_) => { return self.retypes_flt_stat(lex, flaw) }
            KEYWORD::types(TYPE::chr_) => { return self.retypes_chr_stat(lex, flaw) }
            KEYWORD::types(TYPE::bol_) => { return self.retypes_bol_stat(lex, flaw) }
            KEYWORD::types(TYPE::arr_) => { return self.retypes_arr_stat(lex, flaw) }
            KEYWORD::types(TYPE::vec_) => { return self.retypes_vec_stat(lex, flaw) }
            KEYWORD::types(TYPE::seq_) => { return self.retypes_seq_stat(lex, flaw) }
            KEYWORD::types(TYPE::mat_) => { return self.retypes_mat_stat(lex, flaw) }
            KEYWORD::types(TYPE::set_) => { return self.retypes_set_stat(lex, flaw) }
            KEYWORD::types(TYPE::map_) => { return self.retypes_map_stat(lex, flaw) }
            KEYWORD::types(TYPE::axi_) => { return self.retypes_axi_stat(lex, flaw) }
            KEYWORD::types(TYPE::tab_) => { return self.retypes_tab_stat(lex, flaw) }
            KEYWORD::types(TYPE::str_) => { return self.retypes_str_stat(lex, flaw) }
            KEYWORD::types(TYPE::num_) => { return self.retypes_num_stat(lex, flaw) }
            KEYWORD::types(TYPE::ptr_) => { return self.retypes_ptr_stat(lex, flaw) }
            KEYWORD::types(TYPE::err_) => { return self.retypes_err_stat(lex, flaw) }
            KEYWORD::types(TYPE::opt_) => { return self.retypes_opt_stat(lex, flaw) }
            KEYWORD::types(TYPE::nev_) => { return self.retypes_nev_stat(lex, flaw) }
            KEYWORD::types(TYPE::uni_) => { return self.retypes_uni_stat(lex, flaw) }
            KEYWORD::types(TYPE::any_) => { return self.retypes_any_stat(lex, flaw) }
            KEYWORD::types(TYPE::non_) => { return self.retypes_non_stat(lex, flaw) }
            KEYWORD::types(TYPE::nil_) => { return self.retypes_nil_stat(lex, flaw) }
            KEYWORD::types(TYPE::rec_) => { return self.retypes_rec_stat(lex, flaw) }
            KEYWORD::types(TYPE::ent_) => { return self.retypes_ent_stat(lex, flaw) }
            KEYWORD::types(TYPE::blu_) => { return self.retypes_blu_stat(lex, flaw) }
            KEYWORD::types(TYPE::std_) => { return self.retypes_std_stat(lex, flaw) }
            KEYWORD::types(TYPE::loc_) => { return self.retypes_loc_stat(lex, flaw) }
            KEYWORD::types(TYPE::url_) => { return self.retypes_url_stat(lex, flaw) }
            KEYWORD::types(TYPE::blk_) => { return self.retypes_blk_stat(lex, flaw) }
            KEYWORD::types(TYPE::rut_) => { return self.retypes_rut_stat(lex, flaw) }
            KEYWORD::types(TYPE::pat_) => { return self.retypes_pat_stat(lex, flaw) }
            KEYWORD::types(TYPE::gen_) => { return self.retypes_gen_stat(lex, flaw) }
            _ => { return self.retypes_all_stat(lex, flaw) }
        }
    }
    // int
    pub fn retypes_int_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Int)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // flt
    pub fn retypes_flt_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Flt)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // chr
    pub fn retypes_chr_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Chr)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // bol
    pub fn retypes_bol_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Bol)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // arr
    pub fn retypes_arr_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Arr)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // vec
    pub fn retypes_vec_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Vec)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // seq
    pub fn retypes_seq_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Seq)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // mat
    pub fn retypes_mat_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Mat)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // set
    pub fn retypes_set_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Set)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // map
    pub fn retypes_map_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Map)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // axi
    pub fn retypes_axi_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Axi)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // tab
    pub fn retypes_tab_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Tab)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // str
    pub fn retypes_str_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Str)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // num
    pub fn retypes_num_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Num)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // ptr
    pub fn retypes_ptr_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Ptr)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // err
    pub fn retypes_err_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Err)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // opt
    pub fn retypes_opt_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Opt)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // nev
    pub fn retypes_nev_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Nev)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // uni
    pub fn retypes_uni_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Uni)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // any
    pub fn retypes_any_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Any)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // non
    pub fn retypes_non_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Non)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // nil
    pub fn retypes_nil_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Nil)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // rec
    pub fn retypes_rec_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Rec)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // ent
    pub fn retypes_ent_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Ent)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // blu
    pub fn retypes_blu_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Blu)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // std
    pub fn retypes_std_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Std)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // loc
    pub fn retypes_loc_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Loc)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // url
    pub fn retypes_url_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Url)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // blk
    pub fn retypes_blk_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Blk)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // rut
    pub fn retypes_rut_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Rut)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // pat
    pub fn retypes_pat_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Pat)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // gen
    pub fn retypes_gen_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        let typ = Some(Box::new(stat::Type(type_expr::Gen)));
        lex.bump();
        self.temp_go_end_type(lex);
        typ
    }
    // ANY
    pub fn retypes_all_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<stat>> {
        lex.bump();
        self.temp_go_end_type(lex);
        None
    }
    /// TEMPOrARY GO TO END VAR
    pub fn temp_go_end_type(&mut self, lex: &mut lexer::BAG) {
        if matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::squarO_)) {
            let deep = lex.curr().loc().deep() -1;
            while !( ( matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::squarC_)) && lex.curr().loc().deep() == deep ) || lex.curr().key().is_eof() ) {
                lex.bump();
            }
            lex.bump();
        }
    }
}
