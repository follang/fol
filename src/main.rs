#[macro_use]
mod types;
mod syntax;
mod helper;
mod semantic;
mod ast; // Add proper AST module

extern crate regex;
extern crate colored;
extern crate dyn_clone;
extern crate terminal_size;

use crate::syntax::index;
use crate::syntax::lexer;
use crate::syntax::parse;

fn main() {
    println!("=== FOL Compiler with Proper AST ===");

    // Get the file path from command line arguments
    let args: Vec<String> = std::env::args().collect();
    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        "./test/main/main.fol" // Default fallback
    };

    // 1. Original parser output (for comparison)
    println!("\n1. Original Parser Output:");
    let input = index::Input::Path(file_path.to_string(), index::SourceType::File);
    let mut elems = lexer::stage3::Elements::init(&input);
    let parser = parse::Parser::init(&mut elems);
    for e in parser.nodes() { 
        println!("{}\n", e) 
    }

    // 2. New AST Parser Integration
    println!("\n2. New AST Parser (Real Integration):");
    let input2 = index::Input::Path(file_path.to_string(), index::SourceType::File);
    let mut lexed = lexer::stage3::Elements::init(&input2);
    let mut ast_parser = crate::ast::parser::AstParser::new();
    
    match ast_parser.parse(&mut lexed) {
        Ok(ast) => {
            println!("Successfully parsed AST:");
            println!("{:#?}", ast);
            
            // 3. Analyze the AST
            println!("\n3. AST Analysis:");
            if let crate::ast::AstNode::Program { declarations } = &ast {
                println!("Program has {} declarations:", declarations.len());
                for (i, decl) in declarations.iter().enumerate() {
                    match decl {
                        crate::ast::AstNode::VarDecl { name, type_hint, .. } => {
                            println!("  [{}] Variable '{}' of type {:?}", i, name, type_hint);
                        }
                        crate::ast::AstNode::FunDecl { name, params, return_type, .. } => {
                            println!("  [{}] Function '{}' with {} params, returns {:?}", 
                                    i, name, params.len(), return_type);
                        }
                        _ => {
                            println!("  [{}] Other declaration", i);
                        }
                    }
                }
                
                // 4. Type inference demo
                println!("\n4. Type Inference:");
                for (i, decl) in declarations.iter().enumerate() {
                    if let Some(inferred_type) = decl.get_type() {
                        println!("  [{}] Inferred type: {:?}", i, inferred_type);
                    }
                }
            }
        }
        Err(errors) => {
            println!("AST parsing had errors:");
            for (i, error) in errors.iter().enumerate() {
                println!("  [{}] Error: {}", i, error);
            }
        }
    }
    
    // 5. Demonstrate AST utility methods
    println!("\n5. AST Utility Demo:");
    let sample_expr = crate::ast::AstNode::BinaryOp {
        op: crate::ast::BinaryOperator::Add,
        left: Box::new(crate::ast::AstNode::Identifier { name: "x".to_string() }),
        right: Box::new(crate::ast::AstNode::Literal(crate::ast::Literal::Integer(10))),
    };
    println!("Sample expression: {:#?}", sample_expr);
    if let Some(expr_type) = sample_expr.get_type() {
        println!("Expression type: {:?}", expr_type);
    }
}

#[test]
fn it_works() {
    assert_eq!("0", "0")
}
