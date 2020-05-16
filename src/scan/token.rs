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
            KEYWORD::bracket(SYMBOL::curlyO_) => true,
            KEYWORD::bracket(SYMBOL::squarO_) => true,
            KEYWORD::bracket(SYMBOL::roundO_) => true,
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
    endline_(bool),
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
            literal(LITERAL::string_) => write!(f, "{}: {}", "LITERAL", "string"),
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
            bracket(SYMBOL::curlyC_ ) => write!(f, "{}: {}", "SYMBOL", "curlyC"),
            bracket(SYMBOL::curlyO_ ) => write!(f, "{}: {}", "SYMBOL", "curlyO"),
            bracket(SYMBOL::squarC_ ) => write!(f, "{}: {}", "SYMBOL", "squarC"),
            bracket(SYMBOL::squarO_ ) => write!(f, "{}: {}", "SYMBOL", "squarO"),
            bracket(SYMBOL::roundC_ ) => write!(f, "{}: {}", "SYMBOL", "roundC"),
            bracket(SYMBOL::roundO_ ) => write!(f, "{}: {}", "SYMBOL", "roundO"),
            bracket(SYMBOL::angleC_ ) => write!(f, "{}: {}", "SYMBOL", "angleC"),
            bracket(SYMBOL::angleO_ ) => write!(f, "{}: {}", "SYMBOL", "angleO"),
            symbol(SYMBOL::dot_ ) => write!(f, "{}: {}", "SYMBOL", "dot"),
            symbol(SYMBOL::comma_ ) => write!(f, "{}: {}", "SYMBOL", "comma"),
            symbol(SYMBOL::colon_ ) => write!(f, "{}: {}", "SYMBOL", "colon"),
            symbol(SYMBOL::semi_ ) => write!(f, "{}: {}", "SYMBOL", "semi"),
            symbol(SYMBOL::escape_ ) => write!(f, "{}: {}", "SYMBOL", "escape"),
            symbol(SYMBOL::pipe_ ) => write!(f, "{}: {}", "SYMBOL", "pipe"),
            symbol(SYMBOL::equal_ ) => write!(f, "{}: {}", "SYMBOL", "equal"),
            symbol(SYMBOL::greater_ ) => write!(f, "{}: {}", "SYMBOL", "greater"),
            symbol(SYMBOL::less_ ) => write!(f, "{}: {}", "SYMBOL", "less"),
            symbol(SYMBOL::plus_ ) => write!(f, "{}: {}", "SYMBOL", "plus"),
            symbol(SYMBOL::minus_ ) => write!(f, "{}: {}", "SYMBOL", "minus"),
            symbol(SYMBOL::under_ ) => write!(f, "{}: {}", "SYMBOL", "under"),
            symbol(SYMBOL::star_ ) => write!(f, "{}: {}", "SYMBOL", "star"),
            symbol(SYMBOL::home_ ) => write!(f, "{}: {}", "SYMBOL", "home"),
            symbol(SYMBOL::root_ ) => write!(f, "{}: {}", "SYMBOL", "root"),
            symbol(SYMBOL::percent_ ) => write!(f, "{}: {}", "SYMBOL", "percent"),
            symbol(SYMBOL::carret_ ) => write!(f, "{}: {}", "SYMBOL", "carret"),
            symbol(SYMBOL::query_ ) => write!(f, "{}: {}", "SYMBOL", "query"),
            symbol(SYMBOL::bang_ ) => write!(f, "{}: {}", "SYMBOL", "bang"),
            symbol(SYMBOL::and_ ) => write!(f, "{}: {}", "SYMBOL", "and"),
            symbol(SYMBOL::at_ ) => write!(f, "{}: {}", "SYMBOL", "at"),
            symbol(SYMBOL::hash_ ) => write!(f, "{}: {}", "SYMBOL", "hash"),
            symbol(SYMBOL::dollar_ ) => write!(f, "{}: {}", "SYMBOL", "dollar"),
            symbol(SYMBOL::degree_ ) => write!(f, "{}: {}", "SYMBOL", "degree"),
            symbol(SYMBOL::sign_ ) => write!(f, "{}: {}", "SYMBOL", "sign"),
            operator(OPERATOR::ddd_ ) => write!(f, "{}: {}", "OPERATOR", "3dot"),
            operator(OPERATOR::dd_ ) => write!(f, "{}: {}", "OPERATOR", "2dot"),
            operator(OPERATOR::assign_) => write!(f, "{}: {}", "OPERATOR", "assign"),
            operator(OPERATOR::assign2_) => write!(f, "{}: {}", "OPERATOR", "assign2"),
            operator(OPERATOR::flow_) => write!(f, "{}: {}", "OPERATOR", "flow"),
            operator(OPERATOR::flow2_) => write!(f, "{}: {}", "OPERATOR", "flow2"),
            operator(OPERATOR::add_) => write!(f, "{}: {}", "OPERATOR", "add"),
            operator(OPERATOR::subtract_) => write!(f, "{}: {}", "OPERATOR", "subtract"),
            operator(OPERATOR::multiply_) => write!(f, "{}: {}", "OPERATOR", "multiply"),
            operator(OPERATOR::divide_) => write!(f, "{}: {}", "OPERATOR", "divide"),
            operator(OPERATOR::greater_) => write!(f, "{}: {}", "OPERATOR", "greater"),
            operator(OPERATOR::less_) => write!(f, "{}: {}", "OPERATOR", "less"),
            operator(OPERATOR::equal_) => write!(f, "{}: {}", "OPERATOR", "equal"),
            operator(OPERATOR::greatereq_) => write!(f, "{}: {}", "OPERATOR", "greatereq"),
            operator(OPERATOR::lesseq_) => write!(f, "{}: {}", "OPERATOR", "lesseq"),
            operator(OPERATOR::addeq_) => write!(f, "{}: {}", "OPERATOR", "addeq"),
            operator(OPERATOR::subtracteq_) => write!(f, "{}: {}", "OPERATOR", "subtracteq"),
            operator(OPERATOR::multiplyeq_) => write!(f, "{}: {}", "OPERATOR", "multiplyeq"),
            operator(OPERATOR::divideeq_) => write!(f, "{}: {}", "OPERATOR", "divideeq"),
            operator(OPERATOR::shiftleft_) => write!(f, "{}: {}", "OPERATOR", "shiftleft"),
            operator(OPERATOR::shiftright_) => write!(f, "{}: {}", "OPERATOR", "shiftright"),
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
            ident => write!(f, "{}: {}", "IDENT", ""),
            comment => write!(f, "{}: {}", "COMMENT", ""),
            illegal => write!(f, "{}: {}", "ILLEGAL", ""),
            _ => write!(f, "{}: {}", "non-def", "")
        }
    }
}

pub fn get_keyword() -> HashMap<String, KEYWORD> {
    let mut keywords: HashMap<String, KEYWORD> = HashMap::new();
    keywords.insert(String::from("use"), KEYWORD::assign(ASSIGN::use_));
    keywords
}
