use colored::Colorize;
use std::fmt;

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
    ali_,
}

impl fmt::Display for ASSIGN {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t;
        match self {
            ASSIGN::use_ => { t = Some("use".to_string()); },
            ASSIGN::var_ => { t = Some("var".to_string()); },
            ASSIGN::def_ => { t = Some("def".to_string()); },
            ASSIGN::fun_ => { t = Some("fun".to_string()); },
            ASSIGN::pro_ => { t = Some("pro".to_string()); },
            ASSIGN::log_ => { t = Some("log".to_string()); },
            ASSIGN::typ_ => { t = Some("typ".to_string()); },
            ASSIGN::ali_ => { t = Some("ali".to_string()); },
            _ => { t = None },
        };
        write!(f, "{}  {}",
            " ASSIGN   ".black().on_red(),
            match t { 
                Some(val) => { (format!(" {} ", val)).black().on_red().to_string() }, 
                None => "".to_string()
            },
        )
    }
}
