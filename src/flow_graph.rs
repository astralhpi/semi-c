extern crate parser;

use parser::ast;
use parser::ast::Span;
use meta_data::MetaData;
use std::collections::{HashMap};
use error::{syntax_error, Error};
use helper::parse;
use symbol_table::SymbolTable;

type TypeTable = SymbolTable<String, Type>;
type ConvertResult<T> = Result<T, Error>;

#[derive(PartialEq, Clone, Debug)]
pub enum Type {
    Void,
    Char,
    Int,
    Float,
    Pointer(Box<Type>),
    Arrow(Vec<Type>, Box<Type>),
}

impl <'a> From<&'a ast::TypeKind> for Type {
    fn from(type_kind: &ast::TypeKind) -> Type {
        match type_kind {
            &ast::TypeKind::Char => Type::Char,
            &ast::TypeKind::Int => Type::Int,
            &ast::TypeKind::Float => Type::Float,
            &ast::TypeKind::Pointer(ref b) => {
                Type::Pointer(Box::new(Type::from(b.as_ref())))
            },
            &ast::TypeKind::Array(ref b, _) => {
                Type::Pointer(Box::new(Type::from(b.as_ref())))
            }
        }

    }
}

impl <'a> From<&'a ast::ReturnTypeKind> for Type {
    fn from(return_type: &ast::ReturnTypeKind) -> Type {
        match return_type {
            &ast::ReturnTypeKind::Void => Type::Void,
            &ast::ReturnTypeKind::Type(ref t) => Type::from(t)
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Func {
    pub span: Span,
    pub decl: FuncDecl,
    pub body: Option<Box<Node>>
}

#[derive(PartialEq, Clone, Debug)]
pub struct FuncDecl {
    pub span: Span,
    pub name: String,
    pub return_type: Type,
    pub params: Vec<(Type, String)>,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Instruction {
    Printf { args_size: usize },
    LoadString(String),
    LoadChar(char),
    LoadInt(i32),
    FuncCall { name: String, args_size: usize }
}

#[derive(PartialEq, Clone, Debug)]
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

impl TypeTable {
    fn push_funcs(
            &mut self,
            program: &ast::Program) {
        for dcl in program.declines.iter() {
            match dcl {
                &ast::GlobalDclKind::Func(ref func, _) => {
                    let name = func.id.node.to_string();
                    let return_type = Type::from(&func.return_type.node);
                    let params = AstConvert::convert_param_types(&func.param_types);
                    let params = params.into_iter().map(
                        |x: (Type, String)| -> Type {
                            let (t, _) = x;
                            t
                        }).collect();
                    let t = Type::Arrow(params, Box::new(return_type));
                    self.insert(name, t);
                }
                _ => {}
            }
        }
    }

    fn push_params(
            &mut self,
            params: &Vec<(Type, String)>) {

        for &(ref t, ref n) in params {
            self.insert(n.clone(), t.clone());
        }
    }


}

struct AstConvert {}

impl AstConvert {

    fn convert_param_types(param_types: &ast::ParamTypes)
            -> Vec<(Type, String)> {

        let mut result: Vec<(Type, String)> = vec![];
        match &param_types.node {
            &ast::ParamTypesKind::Void => {},
            &ast::ParamTypesKind::Params(ref vec) => {
                for item in vec {
                    let (ref t, ref id) = *item;
                    let t = Type::from(&t.node);
                    result.push((t, id.node.clone()));
                }
            }
        };

        result
    }


    pub fn convert_program(program: &ast::Program)
            -> ConvertResult<HashMap<String, Func>> {

        let mut type_table = TypeTable::new();
        type_table.push_scope();
        type_table.push_funcs(program);

        let mut table: HashMap<String, Func> = HashMap::new();

        for dcl in &program.declines {
            match dcl {
                &ast::GlobalDclKind::Var(ref var_delc) => {
                    return Result::Err(
                        Error::NotImplementedSyntax(var_delc.span.clone()))
                },
                &ast::GlobalDclKind::Func(ref dcl, ref body) => {
                    let return_type = Type::from(&dcl.return_type.node);
                    let name = dcl.id.node.to_string();
                    let params = AstConvert::convert_param_types(
                        &dcl.param_types);


                    type_table.push_scope();
                    type_table.push_params(&params);

                    let decl = FuncDecl {
                        span: dcl.span.clone(),
                        name: name.clone(),
                        return_type,
                        params,
                    };

                    let node = AstConvert::convert_stmts(
                        &body.stmt,
                        &mut type_table,
                        &decl.return_type)?;


                    type_table.drop_scope();

                    let func = Func {
                        span: body.span.clone(),
                        decl: decl,
                        body: match node {
                            Some(n) => Some(Box::new(n)),
                            None => None
                        }
                    };

                    table.insert(name, func);

                }
            }
        }

        Result::Ok(table)
    }

    pub fn convert_stmts(
        stmts: &Vec<ast::Stmt>,
        type_table: &mut TypeTable,
        return_type: &Type) -> ConvertResult<Option<Node>>{

        Ok(Option::None)

    }


}




//pub fn convert_stmt(
//    ast: &ast::Stmt,
//    meta: &MetaData,
//    symbol_table: &SymbolTable<String, Type>) -> Node {
//
//    match &ast.node {
//        _ => panic!("no implementation")
//
//    }
//}
//
//fn some_node(span: Span, eval_type:Type, instruction:Instruction) 
//        -> Result<Node, String> {
//    Result::Ok(Node {
//        span,
//        eval_type,
//        instruction,
//        next: Option::None
//    })
//}

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
fn test_push_funcs() {
    let code = r#"int main(void) {
        printf("Hello Wordl!");
    }"#;
    let meta = MetaData::new(code.to_string());
    let ast = parse(&meta).unwrap();
    let mut table: SymbolTable<String, Type> = SymbolTable::new();
    table.push_scope();
    table.push_funcs(&ast);
    assert_eq!(
        table.get(&"main".to_string()).unwrap(),
        &Type::Arrow(vec![], Box::new(Type::Int)));
}

#[test]
fn simple_program() {
    let code = r#"int main(void) {
        printf("Hello Wordl!");
    }"#;
    let meta = MetaData::new(code.to_string());
    let ast = parse(&meta).unwrap();
    let func_table = AstConvert::convert_program(&ast).unwrap();
    assert_eq!(func_table.len(), 1);
}
