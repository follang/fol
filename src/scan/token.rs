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
    bracket(SYMBOL),
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
            KEYWORD::bracket(_) => true,
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
}


// std::cmp::PartialEq<fn(SYMBOL) -> KEYWORD {KEYWORD::symbol}>


#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LITERAL {
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
    space_,
    endline_{ terminated: bool },
    endfile_,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SYMBOL {
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
    dd_,
    ddd_,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BUILDIN {
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
    blk_
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OPTION {
    mut_,
    imu_,
    sta_,
    rea_,
    exp_,
    nor_,
    hid_,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FORM {
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
            literal(LITERAL::string_) => write!(f, "{: <10} {: <10}", "LITERAL", "string"),
            literal(LITERAL::char_) => write!(f, "{: <10} {: <10}", "LITERAL", "char"),
            literal(LITERAL::float_) => write!(f, "{: <10} {: <10}", "LITERAL", "float"),
            literal(LITERAL::decimal_) => write!(f, "{: <10} {: <10}", "LITERAL", "decimal"),
            literal(LITERAL::hexal_) => write!(f, "{: <10} {: <10}", "LITERAL", "hexal"),
            literal(LITERAL::octal_) => write!(f, "{: <10} {: <10}", "LITERAL", "octal"),
            literal(LITERAL::binary_) => write!(f, "{: <10} {: <10}", "LITERAL", "binary"),
            void(VOID::endline_ { terminated: false } ) => write!(f, "{: <10} {: <10}", "VOID", "eol(nont)"),
            void(VOID::endline_ { terminated: true } ) => write!(f, "{: <10} {: <10}", "VOID", "eol(term)"),
            void(VOID::space_ ) => write!(f, "{: <10} {: <10}", "VOID", "space"),
            void(VOID::endfile_ ) => write!(f, "{: <10} {: <10}", "VOID", "EOF"),
            bracket(SYMBOL::curlyC_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "curlyC"),
            bracket(SYMBOL::curlyO_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "curlyO"),
            bracket(SYMBOL::squarC_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "squarC"),
            bracket(SYMBOL::squarO_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "squarO"),
            bracket(SYMBOL::roundC_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "roundC"),
            bracket(SYMBOL::roundO_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "roundO"),
            bracket(SYMBOL::angleC_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "angleC"),
            bracket(SYMBOL::angleO_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "angleO"),
            symbol(SYMBOL::dot_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "dot"),
            symbol(SYMBOL::comma_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "comma"),
            symbol(SYMBOL::colon_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "colon"),
            symbol(SYMBOL::semi_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "semi"),
            symbol(SYMBOL::escape_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "escape"),
            symbol(SYMBOL::pipe_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "pipe"),
            symbol(SYMBOL::equal_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "equal"),
            symbol(SYMBOL::greater_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "greater"),
            symbol(SYMBOL::less_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "less"),
            symbol(SYMBOL::plus_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "plus"),
            symbol(SYMBOL::minus_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "minus"),
            symbol(SYMBOL::under_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "under"),
            symbol(SYMBOL::star_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "star"),
            symbol(SYMBOL::home_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "home"),
            symbol(SYMBOL::root_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "root"),
            symbol(SYMBOL::percent_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "percent"),
            symbol(SYMBOL::carret_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "carret"),
            symbol(SYMBOL::query_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "query"),
            symbol(SYMBOL::bang_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "bang"),
            symbol(SYMBOL::and_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "and"),
            symbol(SYMBOL::at_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "at"),
            symbol(SYMBOL::hash_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "hash"),
            symbol(SYMBOL::dollar_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "dollar"),
            symbol(SYMBOL::degree_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "degree"),
            symbol(SYMBOL::sign_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "sign"),
            operator(OPERATOR::ddd_ ) => write!(f, "{: <10} {: <10}", "OPERATOR", "3dot"),
            operator(OPERATOR::dd_ ) => write!(f, "{: <10} {: <10}", "OPERATOR", "2dot"),
            assign(ASSIGN::use_) => write!(f, "{: <10} {: <10}", "ASSIGN", "use"),
            assign(ASSIGN::var_) => write!(f, "{: <10} {: <10}", "ASSIGN", "var"),
            assign(ASSIGN::def_) => write!(f, "{: <10} {: <10}", "ASSIGN", "def"),
            assign(ASSIGN::fun_) => write!(f, "{: <10} {: <10}", "ASSIGN", "fun"),
            assign(ASSIGN::pro_) => write!(f, "{: <10} {: <10}", "ASSIGN", "pro"),
            assign(ASSIGN::log_) => write!(f, "{: <10} {: <10}", "ASSIGN", "log"),
            assign(ASSIGN::typ_) => write!(f, "{: <10} {: <10}", "ASSIGN", "typ"),
            assign(ASSIGN::ali_) => write!(f, "{: <10} {: <10}", "ASSIGN", "ali"),
            types(TYPE::int_) => write!(f, "{: <10} {: <10}", "TYPE", "int"),
            types(TYPE::flt_) => write!(f, "{: <10} {: <10}", "TYPE", "flt"),
            types(TYPE::chr_) => write!(f, "{: <10} {: <10}", "TYPE", "chr"),
            types(TYPE::bol_) => write!(f, "{: <10} {: <10}", "TYPE", "bol"),
            types(TYPE::arr_) => write!(f, "{: <10} {: <10}", "TYPE", "arr"),
            types(TYPE::vec_) => write!(f, "{: <10} {: <10}", "TYPE", "vec"),
            types(TYPE::seq_) => write!(f, "{: <10} {: <10}", "TYPE", "seq"),
            types(TYPE::mat_) => write!(f, "{: <10} {: <10}", "TYPE", "mat"),
            types(TYPE::set_) => write!(f, "{: <10} {: <10}", "TYPE", "set"),
            types(TYPE::map_) => write!(f, "{: <10} {: <10}", "TYPE", "map"),
            types(TYPE::axi_) => write!(f, "{: <10} {: <10}", "TYPE", "axi"),
            types(TYPE::tab_) => write!(f, "{: <10} {: <10}", "TYPE", "tab"),
            types(TYPE::str_) => write!(f, "{: <10} {: <10}", "TYPE", "str"),
            types(TYPE::num_) => write!(f, "{: <10} {: <10}", "TYPE", "num"),
            types(TYPE::ptr_) => write!(f, "{: <10} {: <10}", "TYPE", "ptr"),
            types(TYPE::err_) => write!(f, "{: <10} {: <10}", "TYPE", "err"),
            types(TYPE::opt_) => write!(f, "{: <10} {: <10}", "TYPE", "opt"),
            types(TYPE::nev_) => write!(f, "{: <10} {: <10}", "TYPE", "nev"),
            types(TYPE::uni_) => write!(f, "{: <10} {: <10}", "TYPE", "uni"),
            types(TYPE::any_) => write!(f, "{: <10} {: <10}", "TYPE", "any"),
            types(TYPE::non_) => write!(f, "{: <10} {: <10}", "TYPE", "non"),
            types(TYPE::nil_) => write!(f, "{: <10} {: <10}", "TYPE", "nil"),
            types(TYPE::rec_) => write!(f, "{: <10} {: <10}", "TYPE", "rec"),
            types(TYPE::ent_) => write!(f, "{: <10} {: <10}", "TYPE", "ent"),
            types(TYPE::blu_) => write!(f, "{: <10} {: <10}", "TYPE", "blu"),
            types(TYPE::std_) => write!(f, "{: <10} {: <10}", "TYPE", "std"),
            types(TYPE::loc_) => write!(f, "{: <10} {: <10}", "TYPE", "loc"),
            types(TYPE::url_) => write!(f, "{: <10} {: <10}", "TYPE", "url"),
            types(TYPE::blk_) => write!(f, "{: <10} {: <10}", "TYPE", "blk"),
            ident => write!(f, "{: <10} {: <10}", "IDENT", ""),
            comment => write!(f, "{: <10} {: <10}", "COMMENT", ""),
            illegal => write!(f, "{: <10} {: <10}", "ILLEGAL", ""),
            _ => write!(f, "{: <10} {: <10}", "non-def", "")
        }
    }
}

pub fn get_keyword() -> HashMap<String, KEYWORD> {
    let mut keywords: HashMap<String, KEYWORD> = HashMap::new();
    keywords.insert(String::from("use"), KEYWORD::assign(ASSIGN::use_));
    keywords
}
