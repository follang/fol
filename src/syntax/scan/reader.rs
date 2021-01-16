#![allow(dead_code)]

use std::fmt;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use regex::Regex;

use crate::syntax::error::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct READER {
    pub path: String,
    pub name: String,
    pub data: String,
}

impl fmt::Display for READER {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "path: {}\nname: {}",
            self.path, self.name
        )
    }
}

/// Creates an iterator that produces tokens from the input string.
pub fn iteratize(input: &str) -> impl Iterator<Item = READER> + '_ {
    let red = READER::init(&input);
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
    pub fn init(s: &str) -> Vec<Self> {
        let mut vec = Vec::new();
        let e = std::fs::canonicalize(s.to_string())
            .unwrap()
            .as_path()
            .to_str()
            .unwrap()
            .to_string();
        for f in file_list(&e).iter() {
            let name: String = std::fs::canonicalize(&f)
                .unwrap()
                .as_path()
                .to_str()
                .unwrap()
                .to_string()
                .trim_start_matches(&e)
                .to_string();
                // .trim_end_matches(".fol").to_string();
            let path: String = std::fs::canonicalize(&f)
                .unwrap()
                .as_path()
                .to_str()
                .unwrap()
                .to_string();
            let data: String = read_string_file(f).unwrap();
            let reader = READER {
                path,
                name,
                data,
            };
            vec.push(reader);
        }
        return vec;
    }

    pub fn file(s: &str) -> Self {
        let name: String = std::fs::canonicalize(&s)
            .unwrap()
            .as_path()
            .to_str()
            .unwrap()
            .to_string()
            .trim_start_matches(&s)
            .to_string();
            // .trim_end_matches(".fol").to_string();
        let path: String = std::fs::canonicalize(&s)
            .unwrap()
            .as_path()
            .to_str()
            .unwrap()
            .to_string();
        let data: String = read_string_file(s).unwrap();
        Self { path, name, data}
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn data(&self) -> &String {
        &self.data
    }

    pub fn set(&mut self, a: String) {
        self.data = a;
    }
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
            if Regex::new(r"(\.mod)$").unwrap().is_match(&filename) {
                continue;
            }
            let newvec = from_dir(&filepath);
            if let Ok(recvec) = newvec { avec.extend(recvec) }
        } else {
            let filetype: String = path
                .unwrap()
                .path()
                .extension()
                .and_then(OsStr::to_str)
                .unwrap()
                .to_string();
            if filetype != "fol" {
                continue;
            }
            avec.push(filepath);
        }
    }
    if avec.is_empty() { 
        Err(Flaw::GettingNoEntry{ msg: Some("No file found") }.report()) 
    } else { 
        Ok(avec) 
    }
}

pub fn file_list(s: &str) -> Vec<String> {
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
            if Regex::new(r"(\.mod)$").unwrap().is_match(&filename) {
                continue;
            }
            let newvec = file_list(&filepath);
            avec.extend(newvec);
        } else {
            let filetype: String = path
                .unwrap()
                .path()
                .extension()
                .and_then(OsStr::to_str)
                .unwrap()
                .to_string();
            if filetype != "fol" {
                continue;
            }
            avec.push(filepath);
        }
    }
    return avec;
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

