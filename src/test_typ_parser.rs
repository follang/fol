#[macro_use]
mod types;
mod syntax;
mod helper;
mod semantic;
mod ast;

extern crate regex;
extern crate colored;
extern crate dyn_clone;
extern crate terminal_size;

use crate::syntax::index;
use crate::syntax::lexer;
use crate::syntax::parse;

fn main() {
    println!("=== Testing Type Declaration Parser ===");

    let file_path = "./test/test_typ_parser.fol";
    
    // Test with original parser
    println!("\nTesting type declarations with original parser:");
    let input = index::Input::Path(file_path.to_string(), index::SourceType::File);
    let mut elems = lexer::stage3::Elements::init(&input);
    let parser = parse::Parser::init(&mut elems);
    
    println!("Parser errors: {}", parser.errors().len());
    for e in parser.errors() {
        println!("Error: {}", e);
    }
    
    println!("\nParsed nodes:");
    for (i, node) in parser.nodes().iter().enumerate() {
        println!("[{}] {}", i, node);
    }
}