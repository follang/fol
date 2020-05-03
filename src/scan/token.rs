#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_variables)]


use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KEYWORD {
    assign(ASSIGN),
    options(OPTION),
    ident,
    types(TYPE),
    form(FORM),
    literal(LITERAL),
    buildin(BUILDIN),
    comment,
    symbol(SYMBOL),
    operator(SYMBOL),
    bracket(SYMBOL),
    void(VOID),
    illegal
}


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
    intA_,
    int8_,
    int16_,
    int32_,
    int64_,
    untA_,
    unt8_,
    unt16_,
    unt32_,
    unt64_,
    fltA_,
    flt32_,
    flt64_,
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
            symbol(SYMBOL::curlyC_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "curlyC"),
            symbol(SYMBOL::curlyO_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "curlyO"),
            symbol(SYMBOL::squarC_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "squarC"),
            symbol(SYMBOL::squarO_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "squarO"),
            symbol(SYMBOL::roundC_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "roundC"),
            symbol(SYMBOL::roundO_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "roundO"),
            symbol(SYMBOL::angleC_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "angleC"),
            symbol(SYMBOL::angleO_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "angleO"),
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
            assign(ASSIGN::use_) => write!(f, "{: <10} {: <10}", "ASSIGN", "use"),
            assign(ASSIGN::var_) => write!(f, "{: <10} {: <10}", "ASSIGN", "var"),
            ident => write!(f, "{: <10} {: <10}", "IDENT", ""),
            comment => write!(f, "{: <10} {: <10}", "COMMENT", ""),
            illegal => write!(f, "{: <10} {: <10}", "ILLEGAL", ""),
            _ => write!(f, "{: <10} {: <10}", "non-def", "")
        }
    }
}
