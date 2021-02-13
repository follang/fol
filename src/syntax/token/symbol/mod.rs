use colored::Colorize;
use std::fmt;

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
    tik_,
}

impl fmt::Display for SYMBOL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t;
        match self {
            SYMBOL::curlyC_ => { t = Some("}".to_string()); },
            SYMBOL::curlyO_ => { t = Some("{".to_string()); },
            SYMBOL::squarC_ => { t = Some("]".to_string()); },
            SYMBOL::squarO_ => { t = Some("[".to_string()); },
            SYMBOL::roundC_ => { t = Some(")".to_string()); },
            SYMBOL::roundO_ => { t = Some("(".to_string()); },
            SYMBOL::angleC_ => { t = Some(">".to_string()); },
            SYMBOL::angleO_ => { t = Some("<".to_string()); },
            SYMBOL::dot_ => { t = Some(".".to_string()); },
            SYMBOL::comma_ => { t = Some(",".to_string()); },
            SYMBOL::colon_ => { t = Some(":".to_string()); },
            SYMBOL::semi_ => { t = Some(";".to_string()); },
            SYMBOL::escape_ => { t = Some("\\".to_string()); },
            SYMBOL::pipe_ => { t = Some("|".to_string()); },
            SYMBOL::equal_ => { t = Some("=".to_string()); },
            SYMBOL::plus_ => { t = Some("+".to_string()); },
            SYMBOL::minus_ => { t = Some("-".to_string()); },
            SYMBOL::under_ => { t = Some("_".to_string()); },
            SYMBOL::star_ => { t = Some("*".to_string()); },
            SYMBOL::home_ => { t = Some("~".to_string()); },
            SYMBOL::root_ => { t = Some("/".to_string()); },
            SYMBOL::percent_ => { t = Some("%".to_string()); },
            SYMBOL::carret_ => { t = Some("^".to_string()); },
            SYMBOL::query_ => { t = Some("?".to_string()); },
            SYMBOL::bang_ => { t = Some("!".to_string()); },
            SYMBOL::and_ => { t = Some("&".to_string()); },
            SYMBOL::at_ => { t = Some("@".to_string()); },
            SYMBOL::hash_ => { t = Some("#".to_string()); },
            SYMBOL::dollar_ => { t = Some("$".to_string()); },
            SYMBOL::degree_ => { t = Some("°".to_string()); },
            SYMBOL::sign_ => { t = Some("§".to_string()); },
            SYMBOL::tik_ => { t = Some("`".to_string()); },
            _ => { t = None },
        };
        write!(f, "{}:{}",
            " SYMBOL   ".black().on_red(),
            match t { 
                Some(val) => { (format!(" {} ", val)).black().on_red().to_string() }, 
                None => "".to_string()
            },
        )
    }
}
