use std::fmt;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use regex::Regex;
use crate::colored::Colorize;
use crate::types::{Con, Flaw};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Source {
    call: String,
    path: String,
}

impl Source {
    pub fn init(input: &str, file: bool) -> Vec<Self> {
        match source(input, file) {
            Ok(s) => { s }
            Err(e) => { crash!(e) }
        }
    }

    /// getting the full path or relatve path
    pub fn path(&self, abs: bool) -> String {
        if abs { self.path.clone() } else { self.rel_path() }
    }

    fn rel_path(&self) -> String {
        std::fs::canonicalize(&self.path).unwrap()
            .as_path()
            .to_str().unwrap()
            .to_string()
            .trim_start_matches(&self.abs_path())
            .to_string()
    }

    fn abs_path(&self) -> String {
        std::fs::canonicalize(full_path(&self.call).unwrap()).unwrap()
            .as_path()
            .parent().unwrap()
            .to_str().unwrap()
            .to_string()
    }

    pub fn module(&self) -> String {
        std::fs::canonicalize(&self.path).unwrap()
            .as_path()
            .parent().unwrap()
            .to_str().unwrap()
            .to_string()
            .trim_start_matches(&self.abs_path())
            .to_string()
    }

}

impl std::default::Default for Source {
    fn default() -> Self {
        Self {
            call: String::new(),
            path: String::new(),
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "abs_path: {}\nrel_path: {}\nmodule:   {}\n",
            self.path(true), self.path(false), self.module()
        )
    }
}

pub fn from_file(input: &str) -> Con<Vec<Source>> {
    let e = check_validity(input, true)?;
    Ok(vec![Source {
        call: input.to_string(),
        path: full_path(&e)?,
    }])
}

pub fn source(input: &str, file: bool) -> Con<Vec<Source>> {
    let mut vec = Vec::new();
    let e = check_validity(input, file)?;
    if file {
        vec.push( Source {
            call: input.to_string(),
            path: full_path(&e)?,
        });
    } else {
        for f in from_dir(&e)? {
            vec.push( Source {
                call: input.to_string(),
                path: full_path(&f)?,
            } );
        }
    }
    Ok(vec)
}

fn full_path(s: &str) -> Con<String> {
    let e = std::fs::canonicalize(s.to_string()).unwrap()
        .as_path()
        .to_str().unwrap()
        .to_string();
    Ok(e)
}

fn from_dir(s: &str) -> Con<Vec<String>> {
    let paths = std::fs::read_dir(s.to_string()).unwrap();
    let mut avec = Vec::new();
    for path in paths {
        let filepath: String = path.as_ref().unwrap().path().to_str().unwrap().to_string();
        let filename: String = path
            .as_ref().unwrap()
            .file_name()
            .to_str().unwrap()
            .to_string();
        if Path::new(&filepath).is_dir() {
            if Regex::new(r"(\.mod)$").unwrap().is_match(&filename) { continue; }
            if let Ok(recvec) = from_dir(&filepath) { avec.extend(recvec) }
        } else {
            let filetype: String = path.unwrap()
                .path()
                .extension()
                .and_then(OsStr::to_str).unwrap()
                .to_string();
            if filetype != "fol" { continue; }
            avec.push(filepath);
        }
    }
    if avec.is_empty() { 
        let msg = format!("{}", "No file found".red());
        Err( catch!(Flaw::GettingNoEntry{msg: Some(msg)}) )
    } else {
        Ok(avec)
    }
}

fn check_validity(s: &str, file: bool) -> Con<String> {
    let path = std::path::Path::new(s);
    if !path.exists() { 
        let msg = format!("path: {} is not a valid path", s.red());
        return Err( catch!(Flaw::GettingWrongPath{msg: Some(msg)}) );
    };
    if path.is_dir() && !file {
        Ok(full_path(s)?)
    } else if path.is_file() && !file {
        Ok(full_path(&path.parent().unwrap().to_str().unwrap().to_string())?)
    } else if path.is_file() && file {
        Ok(full_path(s)?)
    } else {
        let msg = format!("path: {} is not a valid file", s.red());
        Err( catch!(Flaw::ReadingBadContent{msg: Some(msg)}) )
    }
}

pub fn sources(input: String, file:bool) -> impl Iterator<Item = Source> {
    let red: Vec<Source> = Source::init(&input, file);
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
