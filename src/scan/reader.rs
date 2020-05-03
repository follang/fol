#![allow(dead_code)]

use std::fs;
use std::fmt;
use std::str;
use std::io;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::ffi::OsStr;

extern crate regex;
use regex::Regex;

pub fn file_list(s: &str) -> Vec<String> {
    let paths = fs::read_dir(s.to_string()).unwrap();
    let mut avec = Vec::new();

    for path in paths {
        let filepath: String = path.as_ref().unwrap().path().to_str().unwrap().to_string();
        let filename: String = path.as_ref().unwrap().file_name().to_str().unwrap().to_string();
        // println!("{}", path);
        if Path::new(&filepath).is_dir() {
            if Regex::new(r"(\.mod)$").unwrap().is_match(&filename) { continue }
            let newvec = file_list(&filepath);
            avec.extend(newvec);
        } else {
            let filetype: String = path.unwrap().path().extension().and_then(OsStr::to_str).unwrap().to_string();
            if filetype != "fol" { continue }
            avec.push(filepath);
        }
    }
    return avec
}

pub fn read_vec_file(s: &str) -> Result<Vec<u8>, io::Error> {
    let mut buffer = Vec::new();
    File::open(s)?.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn read_string_file(s: &str) -> Result<String, io::Error> {
    let mut buffer = String::new();
    File::open(s)?.read_to_string(&mut buffer)?;
    Ok(buffer)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct READER {
    pub path: String,
    pub file: String,
    pub name: String,
    pub data: String,
    pub past: String,
}

impl fmt::Display for READER {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "path: {}\nfile: {}\nname: {}",
            self.path, self.file, self.name)
    }
}

// pub struct Mod {
    // pub amod: Vec<READER>,
    // index: usize
// }

// impl Iterator for Mod {
    // type Item = READER;
    // fn next(&mut self) -> Option<Self::Item> {
        // while self.index < self.amod.len() - 1 {
            // let prev = self.amod[self.index].clone();
            // self.index += 1;
            // Some(prev);
        // }
        // None
    // }
// }

/// Creates an iterator that produces tokens from the input string.
pub fn iteratize(input: &str) -> impl Iterator<Item = READER> + '_ {
    let red = READER::init(&input);
    let mut index: usize = 0;
    std::iter::from_fn(move || {
        if index >= red.len() { return None; }
        let prev = red[index].clone();
        index += 1;
        Some(prev)
    })
}

impl READER {
    pub fn init(s: &str) -> Vec<Self> {
        let mut vec = Vec::new();
        let e = fs::canonicalize(s.to_string()).unwrap().as_path().to_str().unwrap().to_string();
        for f in file_list(&e).iter(){
            let file : String = Path::new(&f).file_name()
                .and_then(OsStr::to_str).unwrap().to_string()
                .trim_end_matches(".fol").to_string();
            let name: String = fs::canonicalize(&f).unwrap()
                .as_path().parent().unwrap().to_str().unwrap().to_string()
                .trim_start_matches(&e).to_string()
                .trim_start_matches("/").to_string();
            let path: String = fs::canonicalize(&f).unwrap()
                .as_path().to_str().unwrap().to_string()
                .trim_start_matches(&e).to_string();
            let data: String = read_string_file(f).unwrap();
            let reader = READER{ path, file, name, data, past: String::new() };
            vec.push(reader);
        }
        return vec
    }

    pub fn file(&self) -> &String {
        &self.file
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
