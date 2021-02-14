pub mod source;
pub mod reader;

pub use crate::syntax::index::source::Source;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)] 
pub enum Input {
    Soruce(Source),
    String(String),
}

pub struct Lines {
    lines: Box<dyn Iterator<Item = String>>,
    _source: Option<Source>,
}

impl Lines {
    pub fn source(&self) -> Option<Source> { self._source.clone() }
    pub fn init(input: Input) -> Self {
        let lines: Box<dyn Iterator<Item = String>>;
        let mut _source = None;
        match input.clone() {
            Input::Soruce(s) => { 
                _source = Some(s.clone());
                lines = Box::new(source_lines(&s));
            },
            Input::String(s) => {
                lines = Box::new(string_lines(&s));
            }
        }
        Self {
            lines,
            _source,
        }
    }
}

impl Iterator for Lines {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        self.lines.next()
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
