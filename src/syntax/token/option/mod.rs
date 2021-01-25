use colored::Colorize;
use std::fmt;

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
    ext_,
}

impl fmt::Display for OPTION {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t;
        match self {
            OPTION::imu_ => { t = Some("imu".to_string()); },
            OPTION::mut_ => { t = Some("mut".to_string()); },
            OPTION::sta_ => { t = Some("sta".to_string()); },
            OPTION::nor_ => { t = Some("nor".to_string()); },
            OPTION::exp_ => { t = Some("exp".to_string()); },
            OPTION::hid_ => { t = Some("hid".to_string()); },
            OPTION::stk_ => { t = Some("stk".to_string()); },
            OPTION::hep_ => { t = Some("hep".to_string()); },
            OPTION::ext_ => { t = Some("ext".to_string()); },
            _ => { t = None },
        };
        write!(f, "{}:{}",
            " OPTION   ".black().on_red(),
            match t { 
                Some(val) => { (format!(" {} ", val)).black().on_red().to_string() }, 
                None => "".to_string()
            },
        )
    }
}
