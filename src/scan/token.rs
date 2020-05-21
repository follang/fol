#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_variables)]


use std::fmt;
use std::collections::HashMap;
use colored::Colorize;

// use enumeq::EnumEq;
// #[derive(Clone, Debug, PartialOrd, Ord, EnumEq)]

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KEYWORD {
    assign(ASSIGN),
    option(OPTION),
    ident(String),
    types(TYPE),
    form(FORM),
    literal(LITERAL),
    buildin(BUILDIN),
    comment,
    symbol(SYMBOL),
    operator(OPERATOR),
    void(VOID),
    illegal
}

impl KEYWORD {
    pub fn is_assign(&self) -> bool {
        match *self {
            KEYWORD::assign(_) => true,
            _ => false,
        }
    }
    pub fn is_option(&self) -> bool {
        match *self {
            KEYWORD::option(_) => true,
            _ => false,
        }
    }
    pub fn is_ident(&self) -> bool {
        match *self {
            KEYWORD::ident(_) => true,
            _ => false,
        }
    }
    pub fn is_type(&self) -> bool {
        match *self {
            KEYWORD::types(_) => true,
            _ => false,
        }
    }
    pub fn is_form(&self) -> bool {
        match *self {
            KEYWORD::form(_) => true,
            _ => false,
        }
    }
    pub fn is_literal(&self) -> bool {
        match *self {
            KEYWORD::literal(_) => true,
            _ => false,
        }
    }
    pub fn is_buildin(&self) -> bool {
        match *self {
            KEYWORD::buildin(_) => true,
            _ => false,
        }
    }
    pub fn is_comment(&self) -> bool {
        match *self {
            KEYWORD::comment => true,
            _ => false,
        }
    }
    pub fn is_open_bracket(&self) -> bool {
        match *self {
            KEYWORD::symbol(SYMBOL::curlyO_) => true,
            KEYWORD::symbol(SYMBOL::squarO_) => true,
            KEYWORD::symbol(SYMBOL::roundO_) => true,
            _ => false,
        }
    }
    pub fn is_close_bracket(&self) -> bool {
        match *self {
            KEYWORD::symbol(SYMBOL::curlyC_) => true,
            KEYWORD::symbol(SYMBOL::squarC_) => true,
            KEYWORD::symbol(SYMBOL::roundC_) => true,
            _ => false,
        }
    }
    pub fn is_bracket(&self) -> bool {
        match *self {
            KEYWORD::symbol(SYMBOL::curlyC_) => true,
            KEYWORD::symbol(SYMBOL::squarC_) => true,
            KEYWORD::symbol(SYMBOL::roundC_) => true,
            KEYWORD::symbol(SYMBOL::curlyO_) => true,
            KEYWORD::symbol(SYMBOL::squarO_) => true,
            KEYWORD::symbol(SYMBOL::roundO_) => true,
            _ => false,
        }
    }
    pub fn is_symbol(&self) -> bool {
        if self.is_bracket() {
            false
        }
        else {
            match *self {
                KEYWORD::symbol(_) => true,
                _ => false,
            }
        }
    }
    pub fn is_operator(&self) -> bool {
        match *self {
            KEYWORD::operator(_) => true,
            _ => false,
        }
    }
    pub fn is_void(&self) -> bool {
        match *self {
            KEYWORD::void(_) => true,
            _ => false,
        }
    }
    pub fn is_eof(&self) -> bool {
        match *self {
            KEYWORD::void(VOID::endfile_) => true,
            _ => false,
        }
    }
    pub fn is_space(&self) -> bool {
        match *self {
            KEYWORD::void(VOID::space_) => true,
            _ => false,
        }
    }
    pub fn is_eol(&self) -> bool {
        match *self {
            KEYWORD::void(VOID::endline_) => true,
            _ => false,
        }
    }
    pub fn is_nonterm(&self) -> bool {
        match *self {
            KEYWORD::symbol(SYMBOL::curlyO_) => true,
            KEYWORD::symbol(SYMBOL::squarO_) => true,
            KEYWORD::symbol(SYMBOL::roundO_) => true,
            KEYWORD::symbol(SYMBOL::dot_) => true,
            KEYWORD::symbol(SYMBOL::comma_) => true,
            _ => false,
        }
    }
    pub fn is_terminal(&self) -> bool {
        match *self {
            KEYWORD::void(VOID::endline_) => true,
            KEYWORD::symbol(SYMBOL::semi_) => true,
            _ => false,
        }
    }
    pub fn is_dot(&self) -> bool {
        match *self {
            KEYWORD::symbol(SYMBOL::dot_) => true,
            _ => false,
        }
    }
}


// std::cmp::PartialEq<fn(SYMBOL) -> KEYWORD {KEYWORD::symbol}>


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LITERAL {
    ANY,
    string_,
    char_,
    float_,
    bool_,
    decimal_,
    hexal_,
    octal_,
    binary_
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum VOID {
    ANY,
    space_,
    endline_,
    endfile_,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SYMBOL {
    ANY,
    roundO_,
    roundC_,
    squarO_,
    squarC_,
    curlyO_,
    curlyC_,
    angleO_,
    angleC_,
    dot_,
    comma_,
    colon_,
    semi_,
    escape_,
    pipe_,
    equal_,
    greater_,
    less_,
    plus_,
    minus_,
    under_,
    star_,
    home_,
    root_,
    percent_,
    carret_,
    query_,
    bang_,
    and_,
    at_,
    hash_,
    dollar_,
    degree_,
    sign_,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OPERATOR {
    ANY,
    dd_,
    ddd_,
    assign_,
    assign2_,
    flow_,
    flow2_,
    add_,
    subtract_,
    multiply_,
    divide_,
    greater_,
    less_,
    equal_,
    noteq_,
    greatereq_,
    lesseq_,
    addeq_,
    subtracteq_,
    multiplyeq_,
    divideeq_,
    shiftleft_,
    shiftright_,
}


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BUILDIN {
    ANY,
    not_,
    or_,
    xor_,
    nor_,
    and_,
    nand_,
    as_,
    if_,
    when_,
    loop_,
    is_,
    has_,
    in_,
    case_,
    this_,
    self_,
    break_,
    return_,
    yeild_,
    panic_,
    report_,
    check_,
    assert_,
    where_,
    true_,
    false_,
    each_,
    for_,
    do_,
    go_,
    get_,
    let_
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ASSIGN {
    ANY,
    use_,
    def_,
    var_,
    fun_,
    pro_,
    log_,
    typ_,
    ali_
}

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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OPTION {
    ANY,
    imu_,
    mut_,
    sta_,
    nor_,
    exp_,
    hid_,
    stk_,
    hep_,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FORM {
    ANY,
    i8_,
    i16_,
    i32_,
    i64_,
    ia_,
    u8_,
    u16_,
    u32_,
    u64_,
    ua_,
    f32_,
    f64_,
    fa_,
}


impl fmt::Display for KEYWORD {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        let key = [
            String::from(" ASSIGN "),
            String::from(" OPTION "),
            String::from(" IDENT "),
            String::from(" TYPE "),
            String::from(" FORM "),
            String::from(" LITERAL "),
            String::from(" BUILDIN "),
            String::from(" SYMBOL "),
            String::from(" OPERATOR "),
            String::from(" VOID "),
            String::from(" COMMENT "),
            String::from(" ILLEGAL ")
        ];
        let mid = "| ";
        let pos = " ";
        use KEYWORD::*;
        let value: String = match self {
            literal(LITERAL::string_) => key[5].clone() + &mid + "string" + &pos,
            literal(LITERAL::bool_) => key[5].clone() + &mid + "bool" + &pos,
            literal(LITERAL::char_) => key[5].clone() + &mid + "char" + &pos,
            literal(LITERAL::float_) => key[5].clone() + &mid + "float" + &pos,
            literal(LITERAL::decimal_) => key[5].clone() + &mid + "decimal" + &pos,
            literal(LITERAL::hexal_) => key[5].clone() + &mid + "hexal" + &pos,
            literal(LITERAL::octal_) => key[5].clone() + &mid + "octal" + &pos,
            literal(LITERAL::binary_) => key[5].clone() + &mid + "binary" + &pos,
            void(VOID::endline_ ) => key[9].clone() + &mid + "eol" + &pos,
            void(VOID::space_ ) => key[9].clone() + &mid + "space" + &pos,
            void(VOID::endfile_ ) => key[9].clone() + &mid + "EOF" + &pos,
            symbol(SYMBOL::curlyC_ ) => key[7].clone() + &mid + "}" + &pos,
            symbol(SYMBOL::curlyO_ ) => key[7].clone() + &mid + "{" + &pos,
            symbol(SYMBOL::squarC_ ) => key[7].clone() + &mid + "]" + &pos,
            symbol(SYMBOL::squarO_ ) => key[7].clone() + &mid + "[" + &pos,
            symbol(SYMBOL::roundC_ ) => key[7].clone() + &mid + ")" + &pos,
            symbol(SYMBOL::roundO_ ) => key[7].clone() + &mid + "(" + &pos,
            symbol(SYMBOL::angleC_ ) => key[7].clone() + &mid + ">" + &pos,
            symbol(SYMBOL::angleO_ ) => key[7].clone() + &mid + "<" + &pos,
            symbol(SYMBOL::dot_ ) => key[7].clone() + &mid + "." + &pos,
            symbol(SYMBOL::comma_ ) => key[7].clone() + &mid + "," + &pos,
            symbol(SYMBOL::colon_ ) => key[7].clone() + &mid + ":" + &pos,
            symbol(SYMBOL::semi_ ) => key[7].clone() + &mid + ";" + &pos,
            symbol(SYMBOL::escape_ ) => key[7].clone() + &mid + "\\" + &pos,
            symbol(SYMBOL::pipe_ ) => key[7].clone() + &mid + "|" + &pos,
            symbol(SYMBOL::equal_ ) => key[7].clone() + &mid + "=" + &pos,
            symbol(SYMBOL::greater_ ) => key[7].clone() + &mid + ">" + &pos,
            symbol(SYMBOL::less_ ) => key[7].clone() + &mid + "<" + &pos,
            symbol(SYMBOL::plus_ ) => key[7].clone() + &mid + "+" + &pos,
            symbol(SYMBOL::minus_ ) => key[7].clone() + &mid + "-" + &pos,
            symbol(SYMBOL::under_ ) => key[7].clone() + &mid + "_" + &pos,
            symbol(SYMBOL::star_ ) => key[7].clone() + &mid + "*" + &pos,
            symbol(SYMBOL::home_ ) => key[7].clone() + &mid + "~" + &pos,
            symbol(SYMBOL::root_ ) => key[7].clone() + &mid + "/" + &pos,
            symbol(SYMBOL::percent_ ) => key[7].clone() + &mid + "%" + &pos,
            symbol(SYMBOL::carret_ ) => key[7].clone() + &mid + "^" + &pos,
            symbol(SYMBOL::query_ ) => key[7].clone() + &mid + "?" + &pos,
            symbol(SYMBOL::bang_ ) => key[7].clone() + &mid + "!" + &pos,
            symbol(SYMBOL::and_ ) => key[7].clone() + &mid + "&" + &pos,
            symbol(SYMBOL::at_ ) => key[7].clone() + &mid + "@" + &pos,
            symbol(SYMBOL::hash_ ) => key[7].clone() + &mid + "#" + &pos,
            symbol(SYMBOL::dollar_ ) => key[7].clone() + &mid + "$" + &pos,
            symbol(SYMBOL::degree_ ) => key[7].clone() + &mid + "°" + &pos,
            symbol(SYMBOL::sign_ ) => key[7].clone() + &mid + "§" + &pos,
            operator(OPERATOR::ddd_ ) => key[8].clone() + &mid + ".." + &pos,
            operator(OPERATOR::dd_ ) => key[8].clone() + &mid + ".." + &pos,
            operator(OPERATOR::assign_) => key[8].clone() + &mid + "=" + &pos,
            operator(OPERATOR::assign2_) => key[8].clone() + &mid + ":=" + &pos,
            operator(OPERATOR::flow_) => key[8].clone() + &mid + "=>" + &pos,
            operator(OPERATOR::flow2_) => key[8].clone() + &mid + "->" + &pos,
            operator(OPERATOR::add_) => key[8].clone() + &mid + "+" + &pos,
            operator(OPERATOR::subtract_) => key[8].clone() + &mid + "-" + &pos,
            operator(OPERATOR::multiply_) => key[8].clone() + &mid + "*" + &pos,
            operator(OPERATOR::divide_) => key[8].clone() + &mid + "/" + &pos,
            operator(OPERATOR::greater_) => key[8].clone() + &mid + ">" + &pos,
            operator(OPERATOR::less_) => key[8].clone() + &mid + "<" + &pos,
            operator(OPERATOR::equal_) => key[8].clone() + &mid + "==" + &pos,
            operator(OPERATOR::noteq_) => key[8].clone() + &mid + "!=" + &pos,
            operator(OPERATOR::greatereq_) => key[8].clone() + &mid + ">=" + &pos,
            operator(OPERATOR::lesseq_) => key[8].clone() + &mid + "<=" + &pos,
            operator(OPERATOR::addeq_) => key[8].clone() + &mid + "+=" + &pos,
            operator(OPERATOR::subtracteq_) => key[8].clone() + &mid + "-=" + &pos,
            operator(OPERATOR::multiplyeq_) => key[8].clone() + &mid + "*=" + &pos,
            operator(OPERATOR::divideeq_) => key[8].clone() + &mid + "/=" + &pos,
            operator(OPERATOR::shiftleft_) => key[8].clone() + &mid + "<<" + &pos,
            operator(OPERATOR::shiftright_) => key[8].clone() + &mid + ">>" + &pos,
            assign(ASSIGN::use_) => key[0].clone() + &mid + "use" + &pos,
            assign(ASSIGN::var_) => key[0].clone() + &mid + "var" + &pos,
            assign(ASSIGN::def_) => key[0].clone() + &mid + "def" + &pos,
            assign(ASSIGN::fun_) => key[0].clone() + &mid + "fun" + &pos,
            assign(ASSIGN::pro_) => key[0].clone() + &mid + "pro" + &pos,
            assign(ASSIGN::log_) => key[0].clone() + &mid + "log" + &pos,
            assign(ASSIGN::typ_) => key[0].clone() + &mid + "typ" + &pos,
            assign(ASSIGN::ali_) => key[0].clone() + &mid + "ali" + &pos,
            types(TYPE::int_) => key[3].clone() + &mid + "int" + &pos,
            types(TYPE::flt_) => key[3].clone() + &mid + "flt" + &pos,
            types(TYPE::chr_) => key[3].clone() + &mid + "chr" + &pos,
            types(TYPE::bol_) => key[3].clone() + &mid + "bol" + &pos,
            types(TYPE::arr_) => key[3].clone() + &mid + "arr" + &pos,
            types(TYPE::vec_) => key[3].clone() + &mid + "vec" + &pos,
            types(TYPE::seq_) => key[3].clone() + &mid + "seq" + &pos,
            types(TYPE::mat_) => key[3].clone() + &mid + "mat" + &pos,
            types(TYPE::set_) => key[3].clone() + &mid + "set" + &pos,
            types(TYPE::map_) => key[3].clone() + &mid + "map" + &pos,
            types(TYPE::axi_) => key[3].clone() + &mid + "axi" + &pos,
            types(TYPE::tab_) => key[3].clone() + &mid + "tab" + &pos,
            types(TYPE::str_) => key[3].clone() + &mid + "str" + &pos,
            types(TYPE::num_) => key[3].clone() + &mid + "num" + &pos,
            types(TYPE::ptr_) => key[3].clone() + &mid + "ptr" + &pos,
            types(TYPE::err_) => key[3].clone() + &mid + "err" + &pos,
            types(TYPE::opt_) => key[3].clone() + &mid + "opt" + &pos,
            types(TYPE::nev_) => key[3].clone() + &mid + "nev" + &pos,
            types(TYPE::uni_) => key[3].clone() + &mid + "uni" + &pos,
            types(TYPE::any_) => key[3].clone() + &mid + "any" + &pos,
            types(TYPE::non_) => key[3].clone() + &mid + "non" + &pos,
            types(TYPE::nil_) => key[3].clone() + &mid + "nil" + &pos,
            types(TYPE::rec_) => key[3].clone() + &mid + "rec" + &pos,
            types(TYPE::ent_) => key[3].clone() + &mid + "ent" + &pos,
            types(TYPE::blu_) => key[3].clone() + &mid + "blu" + &pos,
            types(TYPE::std_) => key[3].clone() + &mid + "std" + &pos,
            types(TYPE::loc_) => key[3].clone() + &mid + "loc" + &pos,
            types(TYPE::url_) => key[3].clone() + &mid + "url" + &pos,
            types(TYPE::blk_) => key[3].clone() + &mid + "blk" + &pos,
            types(TYPE::rut_) => key[3].clone() + &mid + "rut" + &pos,
            types(TYPE::pat_) => key[3].clone() + &mid + "pat" + &pos,
            types(TYPE::gen_) => key[3].clone() + &mid + "gen" + &pos,
            buildin(BUILDIN::not_) => key[6].clone() + &mid + "not" + &pos,
            buildin(BUILDIN::or_) => key[6].clone() + &mid + "or" + &pos,
            buildin(BUILDIN::xor_) => key[6].clone() + &mid + "xor" + &pos,
            buildin(BUILDIN::nor_) => key[6].clone() + &mid + "nor" + &pos,
            buildin(BUILDIN::and_) => key[6].clone() + &mid + "and" + &pos,
            buildin(BUILDIN::nand_) => key[6].clone() + &mid + "nand" + &pos,
            buildin(BUILDIN::as_) => key[6].clone() + &mid + "as" + &pos,
            buildin(BUILDIN::if_) => key[6].clone() + &mid + "if" + &pos,
            buildin(BUILDIN::when_) => key[6].clone() + &mid + "when" + &pos,
            buildin(BUILDIN::loop_) => key[6].clone() + &mid + "loop" + &pos,
            buildin(BUILDIN::is_) => key[6].clone() + &mid + "is" + &pos,
            buildin(BUILDIN::has_) => key[6].clone() + &mid + "has" + &pos,
            buildin(BUILDIN::in_) => key[6].clone() + &mid + "in" + &pos,
            buildin(BUILDIN::case_) => key[6].clone() + &mid + "case" + &pos,
            buildin(BUILDIN::this_) => key[6].clone() + &mid + "this" + &pos,
            buildin(BUILDIN::self_) => key[6].clone() + &mid + "self" + &pos,
            buildin(BUILDIN::break_) => key[6].clone() + &mid + "break" + &pos,
            buildin(BUILDIN::return_) => key[6].clone() + &mid + "return" + &pos,
            buildin(BUILDIN::yeild_) => key[6].clone() + &mid + "yeild" + &pos,
            buildin(BUILDIN::panic_) => key[6].clone() + &mid + "panic" + &pos,
            buildin(BUILDIN::report_) => key[6].clone() + &mid + "report" + &pos,
            buildin(BUILDIN::check_) => key[6].clone() + &mid + "check" + &pos,
            buildin(BUILDIN::assert_) => key[6].clone() + &mid + "assert" + &pos,
            buildin(BUILDIN::where_) => key[6].clone() + &mid + "where" + &pos,
            buildin(BUILDIN::true_) => key[6].clone() + &mid + "true" + &pos,
            buildin(BUILDIN::false_) => key[6].clone() + &mid + "false" + &pos,
            buildin(BUILDIN::each_) => key[6].clone() + &mid + "each" + &pos,
            buildin(BUILDIN::for_) => key[6].clone() + &mid + "for" + &pos,
            buildin(BUILDIN::do_) => key[6].clone() + &mid + "do" + &pos,
            buildin(BUILDIN::go_) => key[6].clone() + &mid + "go" + &pos,
            buildin(BUILDIN::get_) => key[6].clone() + &mid + "get" + &pos,
            buildin(BUILDIN::let_) => key[6].clone() + &mid + "let" + &pos,
            form(FORM::i8_) => key[4].clone() + &mid + "i8" + &pos,
            form(FORM::i16_) => key[4].clone() + &mid + "i16" + &pos,
            form(FORM::i32_) => key[4].clone() + &mid + "i32" + &pos,
            form(FORM::i64_) => key[4].clone() + &mid + "i64" + &pos,
            form(FORM::ia_) => key[4].clone() + &mid + "ia" + &pos,
            form(FORM::u8_) => key[4].clone() + &mid + "u8" + &pos,
            form(FORM::u16_) => key[4].clone() + &mid + "u16" + &pos,
            form(FORM::u32_) => key[4].clone() + &mid + "u32" + &pos,
            form(FORM::u64_) => key[4].clone() + &mid + "u64" + &pos,
            form(FORM::ua_) => key[4].clone() + &mid + "ua" + &pos,
            form(FORM::f32_) => key[4].clone() + &mid + "f32" + &pos,
            form(FORM::f64_) => key[4].clone() + &mid + "f64" + &pos,
            form(FORM::fa_) => key[4].clone() + &mid + "fa" + &pos,
            option(OPTION::imu_) => key[1].clone() + &mid + "imu" + &pos,
            option(OPTION::mut_) => key[1].clone() + &mid + "mut" + &pos,
            option(OPTION::sta_) => key[1].clone() + &mid + "sta" + &pos,
            option(OPTION::nor_) => key[1].clone() + &mid + "nor" + &pos,
            option(OPTION::exp_) => key[1].clone() + &mid + "exp" + &pos,
            option(OPTION::hid_) => key[1].clone() + &mid + "hid" + &pos,
            option(OPTION::stk_) => key[1].clone() + &mid + "stk" + &pos,
            option(OPTION::hep_) => key[1].clone() + &mid + "hep" + &pos,
            ident(a) => key[2].clone() + &mid + a + &pos,
            comment => key[10].clone(),
            illegal => key[11].clone(),
            literal(LITERAL::ANY) => key[5].clone(),
            void(VOID::ANY ) => key[9].clone(),
            symbol(SYMBOL::ANY ) => key[7].clone(),
            operator(OPERATOR::ANY ) => key[8].clone(),
            buildin(BUILDIN::ANY) => key[6].clone(),
            assign(ASSIGN::ANY) => key[0].clone(),
            types(TYPE::ANY) => key[3].clone(),
            form(FORM::ANY) => key[4].clone(),
            option(OPTION::ANY) => key[2].clone(),
        };
        write!(f, "{}", value.black().on_white())
    }
}

pub fn get_keyword() -> HashMap<String, KEYWORD> {
    let mut keywords: HashMap<String, KEYWORD> = HashMap::new();
    keywords.insert(String::from("use"), KEYWORD::assign(ASSIGN::use_));
    keywords
}
