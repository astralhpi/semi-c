extern crate parser;

use parser::ast;
use parser::ast::Span;
use meta_data::MetaData;
use std::collections::{HashMap, LinkedList};
use error::{syntax_error, Error};
use helper::parse;
use symbol_table::SymbolTable;

type TypeTable = SymbolTable<String, Type>;
type ConvertResult<T> = Result<T, Error>;
type Flow = LinkedList<Node>;

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
    pub body: LinkedList<Node>
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
    FuncCall { name: String, args_size: usize },
    CharToInt,
    IntToFloat
}

#[derive(PartialEq, Clone, Debug)]
pub struct Node {
    span: Span,
    instruction: Instruction,
}

impl Node {
    pub fn new(span: Span, instruction: Instruction)
            -> Node {
        Node {
            span,
            instruction
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
                    let params = Convert::convert_param_types(&func.param_types);
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

struct Convert {}

impl Convert {

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
                    let params = Convert::convert_param_types(
                        &dcl.param_types);


                    type_table.push_scope();
                    type_table.push_params(&params);

                    let decl = FuncDecl {
                        span: dcl.span.clone(),
                        name: name.clone(),
                        return_type,
                        params,
                    };

                    let flow = Convert::convert_stmts(
                        &body.stmt,
                        &mut type_table,
                        &decl.return_type)?;


                    type_table.drop_scope();

                    let func = Func {
                        span: body.span.clone(),
                        decl: decl,
                        body: flow
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
            return_type: &Type) -> ConvertResult<Flow> {

        let mut result: Flow = Flow::new();

        for stmt in stmts.iter() {

            let mut flow = Convert::convert_stmt(
                stmt, type_table, return_type)?;
            result.append(&mut flow);
        }
        Ok(result)
    }

    pub fn convert_stmt(
            stmt: &ast::Stmt,
            type_table: &mut TypeTable,
            return_type: &Type) -> ConvertResult<Flow> {
        
        match &stmt.node {
            &ast::StmtKind::Call(ref id, ref exprs) => {
                let (flow, t) = Convert::convert_func_call(
                    id.node.to_string(), exprs, &stmt.span, type_table)?;
                Ok(flow)

            },
            _ => Err(Error::NotImplementedSyntax(stmt.span.clone()))
        }
    }
    
    pub fn convert_func_call(func_id: String,
                             args: &Vec<ast::Expr>,
                             span: &Span,
                             type_table: &TypeTable) -> ConvertResult<(Flow, Type)> {
        let mut result = Flow::new();
        if func_id == "printf" {
            if args.len() == 0 {
                return Err(Error::TypeError(span.clone()));
            } else {
                for (i, ref item) in args.iter().enumerate() {
                    let (mut flow, t) = Convert::convert_expr(item, type_table)?;
                    if i == 0 && t != Type::Pointer(Box::new(Type::Char)) {
                        return Err(Error::TypeError(item.span.clone()));
                    }
                    result.append(&mut flow);
                }
                result.push_back(Node {
                    span: span.clone(),
                    instruction: Instruction::Printf {args_size: args.len()}
                });
                return Ok((result, Type::Void));
            }
        } else {
            let err = Error::NotDeclared(span.clone());
            let t = type_table.get(&func_id).ok_or(err)?;
            match t {
                &Type::Arrow(ref arg_types, ref return_type) => {
                    if arg_types.len() != args.len() {
                        return Err(Error::TypeError(span.clone()));
                    }

                    for (i, ref item) in args.iter().enumerate() {
                        let (mut flow, t) = Convert::convert_expr(
                            item, type_table)?;
                        let mut cast = Convert::auto_type_cast(
                            &t, &arg_types[i], span)?;
                        result.append(&mut flow);
                        result.append(&mut cast);
                    }

                    result.push_back(Node {
                        span: span.clone(),
                        instruction: Instruction::FuncCall {
                            name: func_id,
                            args_size: args.len()
                        }
                    });

                    return Ok((result, *return_type.clone()));

                },
                _ => return Err(Error::TypeError(span.clone())),
            }

        }
    }

    pub fn convert_expr(expr: &ast::Expr,
                        type_table: &TypeTable) -> ConvertResult<(Flow, Type)> {
        
        let mut flow = Flow::new();
        match &expr.node {
            &ast::ExprKind::Lit(ref lit) => {
                let span = lit.span.clone();
                match &lit.node {
                    &ast::LitKind::String(ref s) => {
                        flow.push_back(Node::new(
                                lit.span.clone(),
                                Instruction::LoadString(s.to_string())));
                        Ok((flow, Type::Pointer(Box::new(Type::Char))))
                    },
                    &ast::LitKind::Char(ref c) => {
                        flow.push_back(Node::new(
                                lit.span.clone(),
                                Instruction::LoadChar(*c)));
                        Ok((flow, Type::Char))

                    },
                    &ast::LitKind::Int(ref i) => {
                        flow.push_back(Node::new(
                                lit.span.clone(),
                                Instruction::LoadInt(*i)));
                        Ok((flow, Type::Int))
                    },
                }
            },
            _ => Err(Error::NotImplementedSyntax(expr.span.clone()))
        }
    }

    pub fn auto_type_cast(from: &Type,
                          to: &Type,
                          span: &Span) -> ConvertResult<Flow> {
        let mut result = Flow::new();
        if from == to {
            Ok(result)
        } else {
            match (from, to) {
                (&Type::Char, &Type::Int) => {
                    result.push_back(Node {
                        span: span.clone(),
                        instruction: Instruction::CharToInt
                    });
                },
                (&Type::Int, &Type::Float) => {
                    result.push_back(Node {
                        span: span.clone(),
                        instruction: Instruction::IntToFloat
                    });
                },
                (_, _) => return Err(Error::TypeError(span.clone())),
            };
            Ok(result)
        }
    }

}


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
