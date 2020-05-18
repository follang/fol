#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_variables)]


use std::fmt;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KEYWORD {
    assign(ASSIGN),
    option(OPTION),
    ident,
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

// impl PartialEq for KEYWORD {
    // fn eq(&self, other: &Self) -> bool {
        // match &self {
            // other => true,
            // _ => false,
        // }
    // }
// }

// std::cmp::PartialEq<fn(scan::token::SYMBOL) -> scan::token::KEYWORD {scan::token::KEYWORD::symbol}>

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
            KEYWORD::ident => true,
            _ => false,
        }
    }
    pub fn is_types(&self) -> bool {
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
    pub fn is_symbol(&self) -> bool {
        match *self {
            KEYWORD::symbol(_) => true,
            _ => false,
        }
    }
    pub fn is_operator(&self) -> bool {
        match *self {
            KEYWORD::operator(_) => true,
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
    pub fn is_void(&self) -> bool {
        match *self {
            KEYWORD::void(_) => true,
            _ => false,
        }
    }
    pub fn is_illegal(&self) -> bool {
        match *self {
            KEYWORD::illegal => true,
            _ => false,
        }
    }
    pub fn is_eof(&self) -> bool {
        match *self {
            KEYWORD::void(VOID::endfile_) => true,
            _ => false,
        }
    }
    pub fn is_eol(&self) -> bool {
        match *self {
            KEYWORD::void(VOID::endline_(_)) => true,
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
            KEYWORD::void(VOID::endline_(true)) => true,
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


#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum VOID {
    ANY,
    space_,
    endline_(bool),
    endfile_,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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


#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
        use KEYWORD::*;
        match self {
        literal(LITERAL::ANY) => write!(f, "{}", "LITERAL"),
            literal(LITERAL::string_) => write!(f, "{}: {}", "LITERAL", "string"),
            literal(LITERAL::bool_) => write!(f, "{}: {}", "LITERAL", "bool"),
            literal(LITERAL::char_) => write!(f, "{}: {}", "LITERAL", "char"),
            literal(LITERAL::float_) => write!(f, "{}: {}", "LITERAL", "float"),
            literal(LITERAL::decimal_) => write!(f, "{}: {}", "LITERAL", "decimal"),
            literal(LITERAL::hexal_) => write!(f, "{}: {}", "LITERAL", "hexal"),
            literal(LITERAL::octal_) => write!(f, "{}: {}", "LITERAL", "octal"),
            literal(LITERAL::binary_) => write!(f, "{}: {}", "LITERAL", "binary"),
            void(VOID::endline_(false) ) => write!(f, "{}: {}", "VOID", "eol"),
            void(VOID::endline_(true) ) => write!(f, "{}: {}", "VOID", "TERM"),
            void(VOID::space_ ) => write!(f, "{}: {}", "VOID", "space"),
            void(VOID::endfile_ ) => write!(f, "{}: {}", "VOID", "EOF"),
            symbol(SYMBOL::curlyC_ ) => write!(f, "{}: {}", "SYMBOL", "}"),
            symbol(SYMBOL::curlyO_ ) => write!(f, "{}: {}", "SYMBOL", "{"),
            symbol(SYMBOL::squarC_ ) => write!(f, "{}: {}", "SYMBOL", "]"),
            symbol(SYMBOL::squarO_ ) => write!(f, "{}: {}", "SYMBOL", "["),
            symbol(SYMBOL::roundC_ ) => write!(f, "{}: {}", "SYMBOL", ")"),
            symbol(SYMBOL::roundO_ ) => write!(f, "{}: {}", "SYMBOL", "("),
            symbol(SYMBOL::angleC_ ) => write!(f, "{}: {}", "SYMBOL", ">"),
            symbol(SYMBOL::angleO_ ) => write!(f, "{}: {}", "SYMBOL", "<"),
            symbol(SYMBOL::dot_ ) => write!(f, "{}: {}", "SYMBOL", "."),
            symbol(SYMBOL::comma_ ) => write!(f, "{}: {}", "SYMBOL", ","),
            symbol(SYMBOL::colon_ ) => write!(f, "{}: {}", "SYMBOL", ":"),
            symbol(SYMBOL::semi_ ) => write!(f, "{}: {}", "SYMBOL", ";"),
            symbol(SYMBOL::escape_ ) => write!(f, "{}: {}", "SYMBOL", "\\"),
            symbol(SYMBOL::pipe_ ) => write!(f, "{}: {}", "SYMBOL", "|"),
            symbol(SYMBOL::equal_ ) => write!(f, "{}: {}", "SYMBOL", "="),
            symbol(SYMBOL::greater_ ) => write!(f, "{}: {}", "SYMBOL", ">"),
            symbol(SYMBOL::less_ ) => write!(f, "{}: {}", "SYMBOL", "<"),
            symbol(SYMBOL::plus_ ) => write!(f, "{}: {}", "SYMBOL", "+"),
            symbol(SYMBOL::minus_ ) => write!(f, "{}: {}", "SYMBOL", "-"),
            symbol(SYMBOL::under_ ) => write!(f, "{}: {}", "SYMBOL", "_"),
            symbol(SYMBOL::star_ ) => write!(f, "{}: {}", "SYMBOL", "*"),
            symbol(SYMBOL::home_ ) => write!(f, "{}: {}", "SYMBOL", "~"),
            symbol(SYMBOL::root_ ) => write!(f, "{}: {}", "SYMBOL", "/"),
            symbol(SYMBOL::percent_ ) => write!(f, "{}: {}", "SYMBOL", "%"),
            symbol(SYMBOL::carret_ ) => write!(f, "{}: {}", "SYMBOL", "^"),
            symbol(SYMBOL::query_ ) => write!(f, "{}: {}", "SYMBOL", "?"),
            symbol(SYMBOL::bang_ ) => write!(f, "{}: {}", "SYMBOL", "!"),
            symbol(SYMBOL::and_ ) => write!(f, "{}: {}", "SYMBOL", "&"),
            symbol(SYMBOL::at_ ) => write!(f, "{}: {}", "SYMBOL", "@"),
            symbol(SYMBOL::hash_ ) => write!(f, "{}: {}", "SYMBOL", "#"),
            symbol(SYMBOL::dollar_ ) => write!(f, "{}: {}", "SYMBOL", "$"),
            symbol(SYMBOL::degree_ ) => write!(f, "{}: {}", "SYMBOL", "°"),
            symbol(SYMBOL::sign_ ) => write!(f, "{}: {}", "SYMBOL", "§"),
            operator(OPERATOR::ddd_ ) => write!(f, "{}: {}", "OPERATOR", "..."),
            operator(OPERATOR::dd_ ) => write!(f, "{}: {}", "OPERATOR", ".."),
            operator(OPERATOR::assign_) => write!(f, "{}: {}", "OPERATOR", "="),
            operator(OPERATOR::assign2_) => write!(f, "{}: {}", "OPERATOR", ":="),
            operator(OPERATOR::flow_) => write!(f, "{}: {}", "OPERATOR", "=>"),
            operator(OPERATOR::flow2_) => write!(f, "{}: {}", "OPERATOR", "->"),
            operator(OPERATOR::add_) => write!(f, "{}: {}", "OPERATOR", "+"),
            operator(OPERATOR::subtract_) => write!(f, "{}: {}", "OPERATOR", "-"),
            operator(OPERATOR::multiply_) => write!(f, "{}: {}", "OPERATOR", "*"),
            operator(OPERATOR::divide_) => write!(f, "{}: {}", "OPERATOR", "/"),
            operator(OPERATOR::greater_) => write!(f, "{}: {}", "OPERATOR", ">"),
            operator(OPERATOR::less_) => write!(f, "{}: {}", "OPERATOR", "<"),
            operator(OPERATOR::equal_) => write!(f, "{}: {}", "OPERATOR", "=="),
            operator(OPERATOR::noteq_) => write!(f, "{}: {}", "OPERATOR", "!="),
            operator(OPERATOR::greatereq_) => write!(f, "{}: {}", "OPERATOR", ">="),
            operator(OPERATOR::lesseq_) => write!(f, "{}: {}", "OPERATOR", "<="),
            operator(OPERATOR::addeq_) => write!(f, "{}: {}", "OPERATOR", "+="),
            operator(OPERATOR::subtracteq_) => write!(f, "{}: {}", "OPERATOR", "-="),
            operator(OPERATOR::multiplyeq_) => write!(f, "{}: {}", "OPERATOR", "*="),
            operator(OPERATOR::divideeq_) => write!(f, "{}: {}", "OPERATOR", "/="),
            operator(OPERATOR::shiftleft_) => write!(f, "{}: {}", "OPERATOR", "<<"),
            operator(OPERATOR::shiftright_) => write!(f, "{}: {}", "OPERATOR", ">>"),
            assign(ASSIGN::use_) => write!(f, "{}: {}", "ASSIGN", "use"),
            assign(ASSIGN::var_) => write!(f, "{}: {}", "ASSIGN", "var"),
            assign(ASSIGN::def_) => write!(f, "{}: {}", "ASSIGN", "def"),
            assign(ASSIGN::fun_) => write!(f, "{}: {}", "ASSIGN", "fun"),
            assign(ASSIGN::pro_) => write!(f, "{}: {}", "ASSIGN", "pro"),
            assign(ASSIGN::log_) => write!(f, "{}: {}", "ASSIGN", "log"),
            assign(ASSIGN::typ_) => write!(f, "{}: {}", "ASSIGN", "typ"),
            assign(ASSIGN::ali_) => write!(f, "{}: {}", "ASSIGN", "ali"),
            types(TYPE::int_) => write!(f, "{}: {}", "TYPE", "int"),
            types(TYPE::flt_) => write!(f, "{}: {}", "TYPE", "flt"),
            types(TYPE::chr_) => write!(f, "{}: {}", "TYPE", "chr"),
            types(TYPE::bol_) => write!(f, "{}: {}", "TYPE", "bol"),
            types(TYPE::arr_) => write!(f, "{}: {}", "TYPE", "arr"),
            types(TYPE::vec_) => write!(f, "{}: {}", "TYPE", "vec"),
            types(TYPE::seq_) => write!(f, "{}: {}", "TYPE", "seq"),
            types(TYPE::mat_) => write!(f, "{}: {}", "TYPE", "mat"),
            types(TYPE::set_) => write!(f, "{}: {}", "TYPE", "set"),
            types(TYPE::map_) => write!(f, "{}: {}", "TYPE", "map"),
            types(TYPE::axi_) => write!(f, "{}: {}", "TYPE", "axi"),
            types(TYPE::tab_) => write!(f, "{}: {}", "TYPE", "tab"),
            types(TYPE::str_) => write!(f, "{}: {}", "TYPE", "str"),
            types(TYPE::num_) => write!(f, "{}: {}", "TYPE", "num"),
            types(TYPE::ptr_) => write!(f, "{}: {}", "TYPE", "ptr"),
            types(TYPE::err_) => write!(f, "{}: {}", "TYPE", "err"),
            types(TYPE::opt_) => write!(f, "{}: {}", "TYPE", "opt"),
            types(TYPE::nev_) => write!(f, "{}: {}", "TYPE", "nev"),
            types(TYPE::uni_) => write!(f, "{}: {}", "TYPE", "uni"),
            types(TYPE::any_) => write!(f, "{}: {}", "TYPE", "any"),
            types(TYPE::non_) => write!(f, "{}: {}", "TYPE", "non"),
            types(TYPE::nil_) => write!(f, "{}: {}", "TYPE", "nil"),
            types(TYPE::rec_) => write!(f, "{}: {}", "TYPE", "rec"),
            types(TYPE::ent_) => write!(f, "{}: {}", "TYPE", "ent"),
            types(TYPE::blu_) => write!(f, "{}: {}", "TYPE", "blu"),
            types(TYPE::std_) => write!(f, "{}: {}", "TYPE", "std"),
            types(TYPE::loc_) => write!(f, "{}: {}", "TYPE", "loc"),
            types(TYPE::url_) => write!(f, "{}: {}", "TYPE", "url"),
            types(TYPE::blk_) => write!(f, "{}: {}", "TYPE", "blk"),
            types(TYPE::rut_) => write!(f, "{}: {}", "TYPE", "rut"),
            types(TYPE::pat_) => write!(f, "{}: {}", "TYPE", "pat"),
            types(TYPE::gen_) => write!(f, "{}: {}", "TYPE", "gen"),
            buildin(BUILDIN::not_) => write!(f, "{}: {}", "BUILDIN", "not"),
            buildin(BUILDIN::or_) => write!(f, "{}: {}", "BUILDIN", "or"),
            buildin(BUILDIN::xor_) => write!(f, "{}: {}", "BUILDIN", "xor"),
            buildin(BUILDIN::nor_) => write!(f, "{}: {}", "BUILDIN", "nor"),
            buildin(BUILDIN::and_) => write!(f, "{}: {}", "BUILDIN", "and"),
            buildin(BUILDIN::nand_) => write!(f, "{}: {}", "BUILDIN", "nand"),
            buildin(BUILDIN::as_) => write!(f, "{}: {}", "BUILDIN", "as"),
            buildin(BUILDIN::if_) => write!(f, "{}: {}", "BUILDIN", "if"),
            buildin(BUILDIN::when_) => write!(f, "{}: {}", "BUILDIN", "when"),
            buildin(BUILDIN::loop_) => write!(f, "{}: {}", "BUILDIN", "loop"),
            buildin(BUILDIN::is_) => write!(f, "{}: {}", "BUILDIN", "is"),
            buildin(BUILDIN::has_) => write!(f, "{}: {}", "BUILDIN", "has"),
            buildin(BUILDIN::in_) => write!(f, "{}: {}", "BUILDIN", "in"),
            buildin(BUILDIN::case_) => write!(f, "{}: {}", "BUILDIN", "case"),
            buildin(BUILDIN::this_) => write!(f, "{}: {}", "BUILDIN", "this"),
            buildin(BUILDIN::self_) => write!(f, "{}: {}", "BUILDIN", "self"),
            buildin(BUILDIN::break_) => write!(f, "{}: {}", "BUILDIN", "break"),
            buildin(BUILDIN::return_) => write!(f, "{}: {}", "BUILDIN", "return"),
            buildin(BUILDIN::yeild_) => write!(f, "{}: {}", "BUILDIN", "yeild"),
            buildin(BUILDIN::panic_) => write!(f, "{}: {}", "BUILDIN", "panic"),
            buildin(BUILDIN::report_) => write!(f, "{}: {}", "BUILDIN", "report"),
            buildin(BUILDIN::check_) => write!(f, "{}: {}", "BUILDIN", "check"),
            buildin(BUILDIN::assert_) => write!(f, "{}: {}", "BUILDIN", "assert"),
            buildin(BUILDIN::where_) => write!(f, "{}: {}", "BUILDIN", "where"),
            buildin(BUILDIN::true_) => write!(f, "{}: {}", "BUILDIN", "true"),
            buildin(BUILDIN::false_) => write!(f, "{}: {}", "BUILDIN", "false"),
            buildin(BUILDIN::each_) => write!(f, "{}: {}", "BUILDIN", "each"),
            buildin(BUILDIN::for_) => write!(f, "{}: {}", "BUILDIN", "for"),
            buildin(BUILDIN::do_) => write!(f, "{}: {}", "BUILDIN", "do"),
            buildin(BUILDIN::go_) => write!(f, "{}: {}", "BUILDIN", "go"),
            buildin(BUILDIN::get_) => write!(f, "{}: {}", "BUILDIN", "get"),
            buildin(BUILDIN::let_) => write!(f, "{}: {}", "BUILDIN", "let"),
            form(FORM::i8_) => write!(f, "{}: {}", "FORM", "i8"),
            form(FORM::i16_) => write!(f, "{}: {}", "FORM", "i16"),
            form(FORM::i32_) => write!(f, "{}: {}", "FORM", "i32"),
            form(FORM::i64_) => write!(f, "{}: {}", "FORM", "i64"),
            form(FORM::ia_) => write!(f, "{}: {}", "FORM", "ia"),
            form(FORM::u8_) => write!(f, "{}: {}", "FORM", "u8"),
            form(FORM::u16_) => write!(f, "{}: {}", "FORM", "u16"),
            form(FORM::u32_) => write!(f, "{}: {}", "FORM", "u32"),
            form(FORM::u64_) => write!(f, "{}: {}", "FORM", "u64"),
            form(FORM::ua_) => write!(f, "{}: {}", "FORM", "ua"),
            form(FORM::f32_) => write!(f, "{}: {}", "FORM", "f32"),
            form(FORM::f64_) => write!(f, "{}: {}", "FORM", "f64"),
            form(FORM::fa_) => write!(f, "{}: {}", "FORM", "fa"),
            option(OPTION::imu_) => write!(f, "{}: {}", "OPTION", "imu"),
            option(OPTION::mut_) => write!(f, "{}: {}", "OPTION", "mut"),
            option(OPTION::sta_) => write!(f, "{}: {}", "OPTION", "sta"),
            option(OPTION::nor_) => write!(f, "{}: {}", "OPTION", "nor"),
            option(OPTION::exp_) => write!(f, "{}: {}", "OPTION", "exp"),
            option(OPTION::hid_) => write!(f, "{}: {}", "OPTION", "hid"),
            option(OPTION::stk_) => write!(f, "{}: {}", "OPTION", "stk"),
            option(OPTION::hep_) => write!(f, "{}: {}", "OPTION", "hep"),
            ident => write!(f, "{}", "IDENT"),
            comment => write!(f, "{}", "COMMENT"),
            illegal => write!(f, "{}", "ILLEGAL"),
            void(VOID::ANY ) => write!(f, "{}", "VOID"),
            symbol(SYMBOL::ANY ) => write!(f, "{}", "SYMBOL"),
            operator(OPERATOR::ANY ) => write!(f, "{}", "OPERATOR"),
            buildin(BUILDIN::ANY) => write!(f, "{}", "BUILDIN"),
            assign(ASSIGN::ANY) => write!(f, "{}", "ASSIGN"),
            types(TYPE::ANY) => write!(f, "{}", "TYPE"),
            form(FORM::ANY) => write!(f, "{}", "FORM"),
            option(OPTION::ANY) => write!(f, "{}", "OPTION"),
        }
    }
}

pub fn get_keyword() -> HashMap<String, KEYWORD> {
    let mut keywords: HashMap<String, KEYWORD> = HashMap::new();
    keywords.insert(String::from("use"), KEYWORD::assign(ASSIGN::use_));
    keywords
}
