#![allow(dead_code)]

use std::fmt;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use regex::Regex;
use crate::colored::Colorize;
use crate::syntax::error::*;
use crate::types::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Source {
    call: String,
    pub path: String,
}

impl Source {
    fn init(s: &str) -> Con<Vec<Self>> {
        let mut vec = Vec::new();
        let e = full_path(s)?;
        for f in from_dir(&e)? {
            vec.push( Source {
                call: s.to_string(),
                path: full_path(&f)?,
            } );
        }
        Ok(vec)
    }

    /// getting the full path or relatve path
    pub fn path(&self, abs: bool) -> String {
        if abs { self.path.clone() } else { self.rel_path() }
    }

    fn rel_path(&self) -> String {
        let e = std::fs::canonicalize(full_path(&self.call).unwrap()).unwrap().as_path().parent().unwrap().to_str().unwrap().to_string();
        // let e = full_path(&self.call).unwrap();
        std::fs::canonicalize(&self.path).unwrap()
            .as_path()
            .to_str().unwrap()
            .to_string()
            .trim_start_matches(&e)
            .to_string()
    }

    pub fn module(&self) -> String {
        let e = std::fs::canonicalize(full_path(&self.call).unwrap()).unwrap().as_path().parent().unwrap().to_str().unwrap().to_string();
        std::fs::canonicalize(&self.path).unwrap()
            .as_path()
            .parent().unwrap()
            .to_str().unwrap()
            .to_string()
            .trim_start_matches(&e)
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


fn check_file_dir(s: &str) -> Con<String> {
    let path = std::path::Path::new(s);
    if !path.exists() { 
        let msg = format!("path: {} is not a valid path", s.red());
        return catch!(Error::Flaw(Flaw::GettingWrongPath{msg: Some(msg)}));
    };
    if path.is_dir() {
        Ok(full_path(s)?)
    } else if path.is_file() {
        Ok(path.parent().unwrap().to_str().unwrap().to_string())
    } else {
        let msg = format!("path: {} is not a valid file", s.red());
        catch!(Error::Flaw(Flaw::ReadingBadContent{msg: Some(msg)}))
    }
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
        catch!(Error::Flaw(Flaw::GettingNoEntry{msg: Some(msg)}))
    } else {
        Ok(avec)
    }
}

// pub fn read_vec_file(s: &str) -> Result<Vec<u8>, std::io::Error> {
//     let mut buffer = Vec::new();
//     File::open(s)?.read_to_end(&mut buffer)?;
//     Ok(buffer)
// }


fn read_string_file(s: &str) -> Con<String> {
    let mut string = String::new();
    let msg = format!("{}", "Error whil reading file".red());
    match File::open(s) {
        Ok(mut b) => { 
            match b.read_to_string(&mut string) {
                Ok(_) => { 
                    if string.is_empty() { 
                        let msg = format!("{}", "File is empty".red());
                        catch!(Error::Flaw(Flaw::ReadingEmptyFile{msg: Some(msg)}))
                    } else {
                        Ok(string)
                    }
                }
                Err(e) => { catch!(Error::Flaw(Flaw::ReadingEmptyFile{msg: Some(e.to_string())})) }
            }
        }
        Err(e) => { catch!(Error::Flaw(Flaw::ReadingBadContent{msg: Some(e.to_string())})) }
    }
}


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
            match Source::init(&input) {
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
