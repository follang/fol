pub mod source;
pub mod reader;

pub use crate::syntax::index::source::{Source, Sources};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)] 
pub enum Input {
    Soruce(Source),
    String(String)
}

// pub struct Srcs {
    
// }
//
//
//

pub struct Lines {
    lines: Box<dyn Iterator<Item = String>>,
    _input: Input,
}

impl Lines {
    pub fn init(input: Input) -> Self {
        let lines: Box<dyn Iterator<Item = String>>;
        match input.clone() {
            Input::Soruce(s) => { 
                lines = Box::new(source_lines(&s));
            },
            Input::String(s) => {
                lines = Box::new(string_lines(&s));
            }
        }
        Self {
            lines,
            _input: input,
        }
    }
}


pub fn source_lines(src: &Source) -> impl Iterator<Item = String> {
    let mut reader = reader::BufReader::open(src.path(true)).unwrap();
    let mut buffer = String::new();
    std::iter::from_fn(move || {
        if let Some(line) = reader.read_line(&mut buffer) {
            return Some(line.unwrap().clone());
        }
        None
    })
}

pub fn string_lines(src: &String) -> impl Iterator<Item = String> {
    let mut input = src.clone();
    std::iter::from_fn(move || {
        let input_copy = input.clone();
        if input.is_empty() {
            return None;
        }
        let split = input.find('\n').map(|i| i + 1).unwrap_or(input.len());
        let (line, rest) = input_copy.split_at(split);
        input = rest.to_string();
        Some(line.to_string())
    })
}
