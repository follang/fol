use colored::Colorize;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OPERATOR {
    ANY,
    dd_,
    ddd_,
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
    noteq_,
    greatereq_,
    lesseq_,
    addeq_,
    subtracteq_,
    multiplyeq_,
    divideeq_,
    shiftleft_,
    shiftright_,
}

impl fmt::Display for OPERATOR {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t;
        match self {
            OPERATOR::ddd_ => { t = Some("..".to_string()); },
            OPERATOR::dd_ => { t = Some("..".to_string()); },
            OPERATOR::assign2_ => { t = Some(":=".to_string()); },
            OPERATOR::flow_ => { t = Some("=>".to_string()); },
            OPERATOR::flow2_ => { t = Some("->".to_string()); },
            OPERATOR::add_ => { t = Some("+".to_string()); },
            OPERATOR::subtract_ => { t = Some("-".to_string()); },
            OPERATOR::multiply_ => { t = Some("*".to_string()); },
            OPERATOR::divide_ => { t = Some("/".to_string()); },
            OPERATOR::greater_ => { t = Some(">".to_string()); },
            OPERATOR::less_ => { t = Some("<".to_string()); },
            OPERATOR::equal_ => { t = Some("==".to_string()); },
            OPERATOR::noteq_ => { t = Some("!=".to_string()); },
            OPERATOR::greatereq_ => { t = Some(">=".to_string()); },
            OPERATOR::lesseq_ => { t = Some("<=".to_string()); },
            OPERATOR::addeq_ => { t = Some("+=".to_string()); },
            OPERATOR::subtracteq_ => { t = Some("-=".to_string()); },
            OPERATOR::multiplyeq_ => { t = Some("*=".to_string()); },
            OPERATOR::divideeq_ => { t = Some("/=".to_string()); },
            OPERATOR::shiftleft_ => { t = Some("<<".to_string()); },
            OPERATOR::shiftright_ => { t = Some(">>".to_string()); },
            _ => { t = None },
        };
        write!(f, "{}: {}",
            " OPERATOR ".black().on_red(),
            match t { Some(val) => val.to_string(), None => "".to_string() },
        )
    }
}
