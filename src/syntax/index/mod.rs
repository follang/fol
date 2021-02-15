pub mod source;
pub mod reader;

pub use crate::syntax::index::source::Source;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)] 
pub enum Input {
    Source(String, bool),
    String(String),
    SourceAlt(Source),
}

// #[derive(Clone, Debug)] 
pub struct Lines {
    pub lines: Box<dyn Iterator<Item = (String, Option<Source>)>>,
}

impl Lines {
    pub fn init(input: &Input) -> Self {
        let lines: Box<dyn Iterator<Item = (String, Option<Source>)>>;
        match input.clone() {
            Input::SourceAlt(s) => { 
                lines = Box::new(source_lines(&s));
            },
            Input::String(s) => {
                lines = Box::new(string_lines(&s));
            }
            Input::Source(s, b) => {
                lines = Box::new(source_lines2(&s, b));
            }
        }
        Self {
            lines,
        }
    }
}

impl Iterator for Lines {
    type Item = (String, Option<Source>);
    fn next(&mut self) -> Option<Self::Item> {
        self.lines.next()
    }
}

pub fn string_lines(src: &String) -> impl Iterator<Item = (String, Option<Source>)> {
    let mut input = src.clone();
    std::iter::from_fn(move || {
        let input_copy = input.clone();
        if input.is_empty() {
            return None;
        }
        let split = input.find('\n').map(|i| i + 1).unwrap_or(input.len());
        let (line, rest) = input_copy.split_at(split);
        input = rest.to_string();
        Some((line.to_string(), None))
    })
}

pub fn source_lines(src: &Source) -> impl Iterator<Item = (String, Option<Source>)> {
    let source = src.clone();
    let mut reader = reader::BufReader::open(source.path(true)).unwrap();
    let mut buffer = String::new();
    std::iter::from_fn(move || {
        match reader.read_line(&mut buffer) {
            Some(line) => {
                return Some((line.unwrap().clone(), Some(source.clone())));
            }, 
            None => { 
                return None 
            }
        }
    })
}

pub fn sources(src: &String, file: bool) -> impl Iterator<Item = Source> {
    let red: Vec<Source> = Source::init(&src, file);
    let mut index: usize = 0;
    std::iter::from_fn(move || {
        if index >= red.len() {
            return None;
        }
        let prev = red[index].clone();
        index += 1;
        Some(prev)
    })
}

pub fn source_lines2(src: &String, file: bool) -> impl Iterator<Item = (String, Option<Source>)> {
    let mut sources = sources(&src, file);
    let source = sources.next().unwrap();
    let mut reader = reader::BufReader::open(source.path(true)).unwrap();
    let mut buffer = String::new();
    let newline = "\0".to_string();
    std::iter::from_fn(move || {
        match reader.read_line(&mut buffer) {
            Some(line) => {
                return Some((line.unwrap().clone(), Some(source.clone())));
            }, 
            None => { 
                match sources.next() {
                    Some(k) => {
                        reader = reader::BufReader::open(k.path(true)).unwrap();
                        let val = (newline.clone(), Some(k));
                        return Some(val)
                    },
                    None => {
                        return None 
                    }
                }
            }
        }
    })
}

