extern crate parser;

use parser::ast;
use parser::ast::Span;
use meta_data::MetaData;
use std::collections::{HashMap};
use error::syntax_error;
use helper::parse;
use symbol_table::SymbolTable;

#[derive(PartialEq, Clone)]
pub enum Type {
    Void,
    Char,
    Int,
    Float,
    Pointer(Box<Type>),
    Arrow(Box<Type>, Vec<Type>),
}

pub struct Func {
    pub span: Span,
    pub name: String,
    pub return_type: Type,
    pub params: Vec<(Type, String)>,
    pub instructions: Vec<Instruction>,
}

pub enum Instruction {
    Printf { args_size: usize },
    LoadString(String),
    LoadChar(char),
    LoadInt(i32),
    FuncCall { name: String, args_size: usize }
}

pub struct Node {
    span: Span,
    eval_type: Type,
    instruction: Instruction,
    next: Option<Box<Node>>
}

impl Node {
    pub fn new(span: Span, eval_type: Type, instruction: Instruction)
            -> Node {
        Node {
            span,
            eval_type,
            instruction,
            next: Option::None
        }

    }

    pub fn last(&self) -> &Node {
        let mut node = &self;
        loop {
            match node.next() {
                Some(n) => node = node,
                None => return node,
            };
        }

    }

    pub fn next(&self) -> Option<&Node> {
        match &self.next {
            &Some(ref node) => Option::Some(node),
            &None => Option::None
        }
    }
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
                return syntax_error(meta, var_delc.span.lo);
            },
            &ast::GlobalDclKind::Func(ref dcl, ref body) => {
                let return_type = convert_return_type(&dcl.return_type, meta)?;
                let name = dcl.id.node.to_string();
                let params = convert_param_types(&dcl.param_types, meta)?;
                let instructions: Vec<Instruction> = vec![];
                let func = Func {
                    span: dcl.span.clone(),
                    name: name.clone(),
                    return_type,
                    params,
                    instructions
                };

                table.insert(name, func);

            }
        }
    }

    Result::Ok(table)
}

pub fn convert_stmt(
    ast: &ast::Stmt,
    meta: &MetaData,
    symbol_table: &SymbolTable<String, Type>) -> Node {

    match &ast.node {
        _ => panic!("no implementation")

    }
}

fn some_node(span: Span, eval_type:Type, instruction:Instruction) 
        -> Result<Node, String> {
    Result::Ok(Node {
        span,
        eval_type,
        instruction,
        next: Option::None
    })
}

//pub fn convert_expr_vec(
//        ast_vec: &Vec<ast::Expr>,
//        meta: &MetaData,
//        symbol_table: &SymbolTable<String, Type>) -> Result<Node, String> {
//
//    let mut nodes: Vec<Node> = vec![];
//    for ast in ast_vec {
//        let node = convert_expr(ast, meta, symbol_table)?;
//        nodes.push(node);
//    }
//    let first = nodes.pop().expect("ast_vec is empty");
//    let mut cur = &first;
//    loop {
//        match nodes.pop() {
//            Some(node) => {
//                cur.next = Some(Box::new(node));
//                cur = &node;
//            },
//            Node => {
//                return Result::Ok(first);
//            }
//
//        }
//    }
//}
//
//pub fn convert_expr(
//    ast: &ast::Expr,
//    meta: &MetaData,
//    symbol_table: &SymbolTable<String, Type>) -> Result<Node, String> {
//
//    match &ast.node {
//        &ast::ExprKind::Lit(ref lit) => {
//            let span = lit.span.clone();
//            match &lit.node {
//                &ast::LitKind::String(ref s) => some_node(
//                    span,
//                    Type::Pointer(Box::new(Type::Char)),
//                    Instruction::LoadString(s.to_string())),
//                &ast::LitKind::Char(ref c) => some_node(
//                    span,
//                    Type::Char,
//                    Instruction::LoadChar(*c)),
//                &ast::LitKind::Int(ref i) => some_node(
//                    span,
//                    Type::Int,
//                    Instruction::LoadInt(*i)),
//            }
//        },
//        &ast::ExprKind::Call(ref id, ref exprs) => {
//            let span = ast.span.clone();
//            let id = id.node.to_string();
//
//            if id == "printf" {
//                if exprs.len() == 0 {
//                    syntax_error(meta, span.lo)
//                }
//                else {
//                    let mut node = convert_expr_vec(exprs, meta, symbol_table)?;
//                    if node.eval_type != Type::Pointer(Box::new(Type::Char)) {
//                        syntax_error(meta, span.lo)
//                    }
//                    else {
//                        node.last().next = Some(Box::new(Node::new(
//                            span,
//                            Type::Void,
//                            Instruction::Printf {args_size:  exprs.len()})));
//                        Result::Ok(node)
//                    }
//                }
//            }
//            else {
//                match symbol_table.get(&id) {
//                    Some(t) => {
//                        match t {
//                            &Type::Arrow(ref return_type, ref arg_types) => {
//                                if exprs.len() != arg_types.len() {
//                                    return syntax_error(meta, span.lo)
//                                }
//                                let mut node = convert_expr_vec(
//                                        exprs, meta, symbol_table)?;
//                                let mut cur = &node;
//                                for t in arg_types {
//                                    if &cur.eval_type != t {
//                                        return syntax_error(meta, span.lo)
//                                    }
//                                    else {
//                                        cur = cur.next();
//                                    }
//                                }
//
//                                node.last().next = Some(Node::new(
//                                        span,
//                                        return_type.clone(),
//                                        Instruction::FuncCall {
//                                            name: id,
//                                            args_size: arg_types.len()
//                                        }));
//                                node
//
//                            },
//                            _ => syntax_error(meta, span.lo)
//
//                        }
//
//                    },
//                    None => syntax_error(meta, span.lo)
//                }
//            }
//
//        }
//        _ => panic!("no implementatiion")
//    }
//}

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
