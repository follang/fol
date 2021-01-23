use colored::Colorize;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    let_,
}


impl fmt::Display for BUILDIN {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t;
        match self {
            BUILDIN::not_ => { t = Some("not".to_string()); },
            BUILDIN::or_ => { t = Some("or".to_string()); },
            BUILDIN::xor_ => { t = Some("xor".to_string()); },
            BUILDIN::nor_ => { t = Some("nor".to_string()); },
            BUILDIN::and_ => { t = Some("and".to_string()); },
            BUILDIN::nand_ => { t = Some("nand".to_string()); },
            BUILDIN::as_ => { t = Some("as".to_string()); },
            BUILDIN::if_ => { t = Some("if".to_string()); },
            BUILDIN::when_ => { t = Some("when".to_string()); },
            BUILDIN::loop_ => { t = Some("loop".to_string()); },
            BUILDIN::is_ => { t = Some("is".to_string()); },
            BUILDIN::has_ => { t = Some("has".to_string()); },
            BUILDIN::in_ => { t = Some("in".to_string()); },
            BUILDIN::case_ => { t = Some("case".to_string()); },
            BUILDIN::this_ => { t = Some("this".to_string()); },
            BUILDIN::self_ => { t = Some("self".to_string()); },
            BUILDIN::break_ => { t = Some("break".to_string()); },
            BUILDIN::return_ => { t = Some("return".to_string()); },
            BUILDIN::yeild_ => { t = Some("yeild".to_string()); },
            BUILDIN::panic_ => { t = Some("panic".to_string()); },
            BUILDIN::report_ => { t = Some("report".to_string()); },
            BUILDIN::check_ => { t = Some("check".to_string()); },
            BUILDIN::assert_ => { t = Some("assert".to_string()); },
            BUILDIN::where_ => { t = Some("where".to_string()); },
            BUILDIN::true_ => { t = Some("true".to_string()); },
            BUILDIN::false_ => { t = Some("false".to_string()); },
            BUILDIN::each_ => { t = Some("each".to_string()); },
            BUILDIN::for_ => { t = Some("for".to_string()); },
            BUILDIN::do_ => { t = Some("do".to_string()); },
            BUILDIN::go_ => { t = Some("go".to_string()); },
            BUILDIN::get_ => { t = Some("get".to_string()); },
            BUILDIN::let_ => { t = Some("let".to_string()); },
            _ => { t = None },
        };
        write!(f, "{}: {}",
            " BUILDIN  ".black().on_red(),
            match t { 
                Some(val) => { (format!(" {} ", val)).black().on_red().to_string() }, 
                None => "".to_string()
            },
        )
    }
}
