#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_variables)]


use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KEYWORD {
    assign_(ASSIGN),
    options_(OPTION),
    buildin_(BUILDIN),
    types_(TYPE),
    form_(FORM),
    ident_(IDENT),
    symbol_(SYMBOL),
    operator_(SYMBOL),
    void_(VOID),
    encap_(ENCAP),
    digit_(DIGIT),
    illegal_
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DIGIT {
    float_,
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
pub enum IDENT {
    ident_,
    silent_
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ENCAP {
    string_,
    char_,
    comment_,
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
    flt32_,
    flt64_,
}

impl fmt::Display for KEYWORD {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use KEYWORD::*;
        match self {
            assign_(ASSIGN::use_) => write!(f, "{: <10} {: <10}", "ASSIGN", "use"),
            assign_(ASSIGN::var_) => write!(f, "{: <10} {: <10}", "ASSIGN", "var"),
            digit_(DIGIT::decimal_) => write!(f, "{: <10} {: <10}", "DIGIT", "decimal"),
            encap_(ENCAP::string_) => write!(f, "{: <10} {: <10}", "ENCAP", "string"),
            void_(VOID::endline_ { terminated: false } ) => write!(f, "{: <10} {: <10}", "VOID", "eol(nont)"),
            void_(VOID::endline_ { terminated: true } ) => write!(f, "{: <10} {: <10}", "VOID", "eol(term)"),
            void_(VOID::space_ ) => write!(f, "{: <10} {: <10}", "VOID", "space"),
            symbol_(SYMBOL::curlyBC_ ) => write!(f, "{: <10} {: <10}", "SYMBOL", "{"),
            ident_(IDENT::ident_ ) => write!(f, "{: <10} {: <10}", "IDENT", "ident"),
            _ => write!(f, "{: <10} {: <10}", "ILLEGAL", "")
        }
    }
}
