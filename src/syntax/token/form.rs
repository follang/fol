use colored::Colorize;
use std::fmt;

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

impl fmt::Display for FORM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t;
        match self {
            FORM::i8_ => { t = Some("i8".to_string()); },
            FORM::i16_ => { t = Some("i16".to_string()); },
            FORM::i32_ => { t = Some("i32".to_string()); },
            FORM::i64_ => { t = Some("i64".to_string()); },
            FORM::ia_ => { t = Some("ia".to_string()); },
            FORM::u8_ => { t = Some("u8".to_string()); },
            FORM::u16_ => { t = Some("u16".to_string()); },
            FORM::u32_ => { t = Some("u32".to_string()); },
            FORM::u64_ => { t = Some("u64".to_string()); },
            FORM::ua_ => { t = Some("ua".to_string()); },
            FORM::f32_ => { t = Some("f32".to_string()); },
            FORM::f64_ => { t = Some("f64".to_string()); },
            FORM::fa_ => { t = Some("fa".to_string()); },
            _ => { t = None },
        };
        write!(f, "{}: {}",
            " FORM    ".black().on_red(),
            match t { 
                Some(val) => { (format!(" {} ", val)).black().on_red().to_string() }, 
                None => "".to_string()
            },
        )
    }
}
