extern crate parser;

use parser::ast;

pub enum Type {
    Char,
    Int,
    Float,
    Pointer(Box<Type>)
}

pub struct Func {
    line: usize,
    return_type: Type,
    param_type: Vec<(Type, String)>

}

pub fn convert_ast(
    ast: ast::Program, line_break_pos: &Vec<usize>) {


}
