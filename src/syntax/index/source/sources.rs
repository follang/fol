use crate::types::Con;
use crate::colored::Colorize;
use crate::syntax::index::source::Source;
use crate::types::error::Flaw;

pub struct Sources {
    srcs:  Box<dyn Iterator<Item = Source>>,
    src: Source,
}

impl Sources {
    pub fn init(dir: String) -> Self {
        let srcs = Box::new(sources(dir));
        let src = Source::default();
        Self { srcs, src }
    }
    pub fn curr(&self) -> Source {
        self.src.clone()
    }
    pub fn bump(&mut self) -> Option<Source> {
        if let Some(v) = self.srcs.next() {
            self.src = v.clone();
            return Some(v)
        }
        None
    }
}

impl Iterator for Sources {
    type Item = Source;
    fn next(&mut self) -> Option<Source> {
        match self.bump() {
            Some(v) => Some(v),
            None => None
        }
    }
        }


pub fn sources(input: String) -> impl Iterator<Item = Source> {
    let red: Vec<Source>;
    match check_file_dir(&input) {
        Ok(input) => {
            match Source::from_folder(&input) {
                Ok(files) => { red = files }
                Err(e) => { crash!(e) }
            }
        }
        Err(e) => { crash!(e) }
    }
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

fn check_file_dir(s: &str) -> Con<String> {
    let path = std::path::Path::new(s);
    if !path.exists() { 
        let msg = format!("path: {} is not a valid path", s.red());
        return Err( catch!(Flaw::GettingWrongPath{msg: Some(msg)}) );
    };
    if path.is_dir() {
        Ok(full_path(s)?)
    } else if path.is_file() {
        Ok(path.parent().unwrap().to_str().unwrap().to_string())
    } else {
        let msg = format!("path: {} is not a valid file", s.red());
        Err( catch!(Flaw::ReadingBadContent{msg: Some(msg)}) )
    }
}

fn full_path(s: &str) -> Con<String> {
    let e = std::fs::canonicalize(s.to_string()).unwrap()
        .as_path()
        .to_str().unwrap()
        .to_string();
    Ok(e)
}
