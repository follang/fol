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
    roundBO_,
    roundBC_,
    squarBO_,
    squarBC_,
    curlyBO_,
    curlyBC_,
    angleBO_,
    angleBC_,
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
    degree_,
    carret_,
    query_,
    bang_,
    and_,
    at_,
    hash_,
    dollar_,
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
    rac_,
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
            literal(LITERAL::binary_) => write!(f, "{: <10} {: <10}", "LITERAL", "binary_"),
            void(VOID::endline_ { terminated: false } ) => write!(f, "{: <10} {: <10}", "VOID", "eol(nont)"),
            void(VOID::endline_ { terminated: true } ) => write!(f, "{: <10} {: <10}", "VOID", "eol(term)"),
            void(VOID::space_ ) => write!(f, "{: <10} {: <10}", "VOID", "space"),
            void(VOID::endfile_ ) => write!(f, "{: <10} {: <10}", "VOID", "EOF"),
            symbol(SYMBOL::curlyBC_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "curlyBC_"),
            symbol(SYMBOL::curlyBO_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "curlyBO_"),
            symbol(SYMBOL::squarBC_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "squarBC_"),
            symbol(SYMBOL::squarBO_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "squarBO_"),
            symbol(SYMBOL::roundBC_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "roundBC_"),
            symbol(SYMBOL::roundBO_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "roundBO_"),
            symbol(SYMBOL::angleBC_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "angleBC_"),
            symbol(SYMBOL::angleBO_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "angleBO_"),
            symbol(SYMBOL::dot_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "dot_"),
            symbol(SYMBOL::comma_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "comma_"),
            symbol(SYMBOL::colon_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "colon_"),
            symbol(SYMBOL::semi_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "semi_"),
            symbol(SYMBOL::escape_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "escape_"),
            symbol(SYMBOL::pipe_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "pipe_"),
            symbol(SYMBOL::equal_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "equal_"),
            symbol(SYMBOL::greater_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "greater_"),
            symbol(SYMBOL::less_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "less_"),
            symbol(SYMBOL::plus_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "plus_"),
            symbol(SYMBOL::minus_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "minus_"),
            symbol(SYMBOL::under_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "under_"),
            symbol(SYMBOL::star_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "star_"),
            symbol(SYMBOL::home_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "home_"),
            symbol(SYMBOL::root_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "root_"),
            symbol(SYMBOL::percent_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "percent_"),
            symbol(SYMBOL::degree_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "degree_"),
            symbol(SYMBOL::carret_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "carret_"),
            symbol(SYMBOL::query_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "query_"),
            symbol(SYMBOL::bang_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "bang_"),
            symbol(SYMBOL::and_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "and_"),
            symbol(SYMBOL::at_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "at_"),
            symbol(SYMBOL::hash_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "hash_"),
            symbol(SYMBOL::dollar_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "dollar_"),
            assign(ASSIGN::use_) => write!(f, "{: <10} {: <10}", "ASSIGN", "use"),
            assign(ASSIGN::var_) => write!(f, "{: <10} {: <10}", "ASSIGN", "var"),
            ident => write!(f, "{: <10} {: <10}", "IDENT", ""),
            comment => write!(f, "{: <10} {: <10}", "COMMENT", ""),
            illegal => write!(f, "{: <10} {: <10}", "ILLEGAL", ""),
            _ => write!(f, "{: <10} {: <10}", "non-def", "")
        }
    }
}
