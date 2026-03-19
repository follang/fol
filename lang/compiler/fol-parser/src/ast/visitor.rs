use super::node::AstNode;
use super::options::{BinaryOperator, FunOption, Literal, TypeOption, UnaryOperator, VarOption};
use super::types::{FolType, LoopCondition, Parameter, TypeDefinition, WhenCase};

/// Visitor pattern for AST traversal and analysis
pub trait AstVisitor {
    fn visit(&mut self, node: &AstNode);

    fn visit_var_decl(
        &mut self,
        _options: &[VarOption],
        _name: &str,
        _type_hint: &Option<FolType>,
        _value: &Option<Box<AstNode>>,
    ) {
    }
    fn visit_fun_decl(
        &mut self,
        _options: &[FunOption],
        _name: &str,
        _params: &[Parameter],
        _return_type: &Option<FolType>,
        _body: &[AstNode],
    ) {
    }
    fn visit_pro_decl(
        &mut self,
        _options: &[FunOption],
        _name: &str,
        _params: &[Parameter],
        _return_type: &Option<FolType>,
        _body: &[AstNode],
    ) {
    }
    fn visit_type_decl(
        &mut self,
        _options: &[TypeOption],
        _name: &str,
        _type_def: &TypeDefinition,
    ) {
    }
    fn visit_binary_op(&mut self, _op: &BinaryOperator, _left: &AstNode, _right: &AstNode) {}
    fn visit_unary_op(&mut self, _op: &UnaryOperator, _operand: &AstNode) {}
    fn visit_function_call(&mut self, _name: &str, _args: &[AstNode]) {}
    fn visit_identifier(&mut self, _name: &str) {}
    fn visit_literal(&mut self, _literal: &Literal) {}
    fn visit_assignment(&mut self, _target: &AstNode, _value: &AstNode) {}
    fn visit_when(
        &mut self,
        _expr: &AstNode,
        _cases: &[WhenCase],
        _default: &Option<Vec<AstNode>>,
    ) {
    }
    fn visit_loop(&mut self, _condition: &LoopCondition, _body: &[AstNode]) {}
    fn visit_block(&mut self, _statements: &[AstNode]) {}
    fn visit_program(&mut self, _declarations: &[AstNode]) {}
}
