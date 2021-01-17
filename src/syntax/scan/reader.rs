#![allow(dead_code)]

use std::fmt;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use regex::Regex;
use crate::colored::Colorize;
use crate::syntax::error::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct READER {
    call: String,
    pub path: String,
    pub data: String,
}

impl fmt::Display for READER {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "abs_path: {}\nrel_path: {}",
            self.path(true), self.path(false)
        )
    }
}

/// Creates an iterator that produces tokens from the input string.
pub fn iteratize(input: &str) -> impl Iterator<Item = READER> + '_ {
    let red: Vec<READER>;
    match READER::init(&input) {
        Ok(files) => { red = files }
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

impl READER {
    pub fn init(s: &str) -> Cont<Vec<Self>> {
        let mut vec = Vec::new();
        let e = full_path(s)?;
        println!("----");
        let pathvec = from_dir(&e)?;
        for f in pathvec.iter() {
            let path = full_path(f)?;
            let data: String = read_string_file(f).unwrap();
            let reader = READER {
                call: s.to_string(),
                path,
                data,
            };
            vec.push(reader);
        }
        Ok(vec)
    }

    /// getting the full path or relatve path
    pub fn path(&self, abs: bool) -> String {
        if abs { self.path.clone() } else { self.rel_path() }
    }

    fn rel_path(&self) -> String {
        let e = std::fs::canonicalize(&self.call)
            .unwrap()
            .as_path()
            .to_str()
            .unwrap()
            .to_string();
        std::fs::canonicalize(&self.path)
            .unwrap()
            .as_path()
            .to_str()
            .unwrap()
            .to_string()
            .trim_start_matches(&e)
            .to_string()
    }

    pub fn data(&self) -> &String {
        &self.data
    }

    pub fn set(&mut self, a: String) {
        self.data = a;
    }
}

fn full_path(s: &str) -> Cont<String> {
    let e = std::fs::canonicalize(s.to_string())
        .unwrap()
        .as_path()
        .to_str()
        .unwrap()
        .to_string();
    Ok(e)
}

pub fn from_dir(s: &str) -> Cont<Vec<String>> {
    let paths = std::fs::read_dir(s.to_string()).unwrap();
    let mut avec = Vec::new();
    for path in paths {
        let filepath: String = path.as_ref().unwrap().path().to_str().unwrap().to_string();
        let filename: String = path
            .as_ref()
            .unwrap()
            .file_name()
            .to_str()
            .unwrap()
            .to_string();
        if Path::new(&filepath).is_dir() {
            if Regex::new(r"(\.mod)$").unwrap().is_match(&filename) { continue; }
            if let Ok(recvec) = from_dir(&filepath) { avec.extend(recvec) }
        } else {
            let filetype: String = path
                .unwrap()
                .path()
                .extension()
                .and_then(OsStr::to_str)
                .unwrap()
                .to_string();
            if filetype != "fol" { continue; }
            avec.push(filepath);
        }
    }
    if avec.is_empty() { 
        let msg = format!("{}", "No file found".red());
        Err( glitch!(Flaw::GettingNoEntry{msg: Some(msg)}) )
    } else {
        Ok(avec)
    }
}

pub fn read_vec_file(s: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut buffer = Vec::new();
    File::open(s)?.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn read_string_file(s: &str) -> Result<String, std::io::Error> {
    let mut buffer = String::new();
    File::open(s)?.read_to_string(&mut buffer)?;
    Ok(buffer)
}

