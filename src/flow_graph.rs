extern crate parser;

use parser::ast;
use parser::ast::Span;
use meta_data::MetaData;
use std::collections::{HashMap};
use error::syntax_error;
use helper::parse;


pub enum Type {
    Void,
    Char,
    Int,
    Float,
    Pointer(Box<Type>)
}

pub struct Func {
    span: Span,
    name: String,
    return_type: Type,
    params: Vec<(Type, String)>

}

pub fn convert_type(ast: &ast::TypeKind, meta: &MetaData) -> Type {
    match ast {
        &ast::TypeKind::Char => Type::Char,
        &ast::TypeKind::Int => Type::Int,
        &ast::TypeKind::Float => Type::Float,
        &ast::TypeKind::Pointer(ref b) => {
            Type::Pointer(Box::new(convert_type(b.as_ref(), meta)))
        },
        &ast::TypeKind::Array(ref b, _) => {
            Type::Pointer(Box::new(convert_type(b.as_ref(), meta)))
        }
    }
}

pub fn convert_return_type(
        ast: &ast::ReturnType,
        meta: &MetaData) -> Result<Type, String> {
    match &ast.node {
        &ast::ReturnTypeKind::Void => Result::Ok(Type::Void),
        &ast::ReturnTypeKind::Type(ref t) => {
            Result::Ok(convert_type(&t, meta))
        }
    }
}

pub fn convert_param_types(
    ast: &ast::ParamTypes,
    meta: &MetaData) -> Result<Vec<(Type, String)>, String> {
    let mut result: Vec<(Type, String)> = vec![];
    match &ast.node {
        &ast::ParamTypesKind::Void => {},
        &ast::ParamTypesKind::Params(ref vec) => {
            for item in vec {
                let (ref t, ref id) = *item;
                result.push((convert_type(&t.node, meta), id.node.clone()));
            }
        }
    };

    Result::Ok(result)
}

pub fn convert_ast(
        ast: &ast::Program,
        meta: &MetaData) -> Result<HashMap<String, Func>, String> {

    let mut table: HashMap<String, Func>  = HashMap::new();
    for dcl in &ast.declines {
        match dcl {
            &ast::GlobalDclKind::Var(ref var_delc) => {
                return Result::Err(syntax_error(meta, var_delc.span.lo))
            },
            &ast::GlobalDclKind::Func(ref dcl, ref body) => {
                let return_type = convert_return_type(&dcl.return_type, meta)?;
                let name = dcl.id.node.to_string();
                let params = convert_param_types(&dcl.param_types, meta)?;
                let func = Func {
                    span: dcl.span.clone(),
                    name: name.clone(),
                    return_type,
                    params
                };

                table.insert(name, func);

            }
        }
    }

    Result::Ok(table)

}

#[test]
fn simple_program() {
    let code = r#"int main(void) {
        printf("Hello Wordl!");
    }"#;
    let meta = MetaData::new(code.to_string());
    let ast = parse(&meta).unwrap();
    let func_table = convert_ast(&ast, &meta).unwrap();
    assert_eq!(1, func_table.len());
}
