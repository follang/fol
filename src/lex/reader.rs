#![allow(dead_code)]

use std::fs;
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
        let filepath: String = path.unwrap().path().to_str().unwrap().to_string();
        let filename: String = Path::new(&filepath).file_name().and_then(OsStr::to_str).unwrap().to_string();
        if Regex::new(r"(\.mod)$").unwrap().is_match(&filename) { continue }
        if Path::new(&filepath).is_dir() {
            let newvec = file_list(&filepath);
            avec.extend(newvec);
        } else {
            let filetype: String = Path::new(&filepath).extension().and_then(OsStr::to_str).unwrap().to_string();
            if filetype != "fol" { continue }
            avec.push(filepath);
        }
    }
    return avec
}

pub fn read_file(s: &str) -> Result<Vec<u8>, io::Error> {
    let mut buffer = Vec::new();
    File::open(s)?.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn in_module(s: &str, p: &str) -> String {
    let entry = fs::canonicalize(s).unwrap().as_path().parent().unwrap().to_str().unwrap().to_string();
    let trimed = entry.trim_start_matches(p).to_string();
    let trimed = trimed.trim_start_matches("/").to_string();
    return trimed
}

pub struct READER {
    path: String,
    file: String,
    name: String,
    data: Vec<u8>,
}

impl READER {
    pub fn init(s: &str) -> Vec<Self> {
        let mut vec = Vec::new();
        let e = fs::canonicalize(s.to_string()).unwrap().as_path().to_str().unwrap().to_string();
        for f in file_list(&e).iter(){
            let path: String = f.to_string();
            let file: String = Path::new(&f).file_name().and_then(OsStr::to_str).unwrap().to_string();
            let name: String = in_module(&f, &e);
            let data: Vec<u8> = read_file(f).unwrap();
            let reader = READER{ path, file, name, data };
            vec.push(reader);
        }
        return vec
    }

    pub fn path(&self) -> &String {
        &self.path
    }

    pub fn file(&self) -> &String {
        &self.file
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }
}
