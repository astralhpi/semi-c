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
    pub body: Vec<Node>
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
    // function call
    Printf { args_size: usize },
    FuncCall { name: String, args_size: usize },

    // load
    LoadString(String),
    LoadChar(char),
    LoadInt(i32),
    LoadFloat(f32),

    // load variable
    LoadVar(String),

    // 1 -> 1
    SaveVar(String, Type),

    // 2 -> 1
    SaveAddr(Type),
    Declare(String, Type),

    StackAlloc,

    // convert
    CharToInt,
    IntToFloat,
    FloatToInt,

    // unary op
    Not,
    Minusi,
    Minusf,
    
    // bin op
    Addi,
    Addf,
    Subi,
    Subf,
    Muli,
    Mulf,
    Divi,
    Divf,
    And,
    Or,
    Eqi,
    Eqf,
    Lti,
    Ltf,
    Gti,
    Gtf,

    // control
    Return,
    ReturnVoid,
    Pop,
    ScopeBegin,
    ScopeEnd,
    
    // move
    Jump(i32),
    JumpIfZero(i32),

}

#[derive(PartialEq, Clone, Debug)]
pub struct Node {
    pub span: Span,
    pub instruction: Instruction,
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
    fn can_declare(&self, name: &str) -> bool {
        let scope = self.list.front();
        match scope {
            None => false,
            Some(ref table) => {
                !table.contains_key(name)
            }
        }
    }
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

pub struct Convert {}

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

                    let mut flow = Convert::convert_stmts(
                        &body.stmt,
                        &mut type_table,
                        &decl.return_type)?;
                    flow.push_back(Node {
                        span: dcl.span.clone(),
                        instruction: Instruction::ReturnVoid
                    });


                    type_table.drop_scope();

                    let mut body_vec: Vec<Node> = vec![];

                    loop {
                        let node = flow.pop_front();
                        match node {
                            Some(n) => body_vec.push(n),
                            None => break
                        };
                    }

                    let func = Func {
                        span: body.span.clone(),
                        decl: decl,
                        body: body_vec
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
                let (mut flow, t) = Convert::convert_func_call(
                    id.node.to_string(), exprs, &stmt.span, type_table)?;
                flow.push_back(Node {
                    span: stmt.span.clone(),
                    instruction: Instruction::Pop,
                });
                Ok(flow)

            },
            &ast::StmtKind::Return(ref b) => {
                match b {
                    &Some(ref e) => {
                        let (mut flow, t) = Convert::convert_expr(e, type_table)?;
                        if return_type != &t {
                            Err(Error::TypeError(stmt.span.clone()))
                        } else {
                            flow.push_back(Node {
                                span: stmt.span.clone(),
                                instruction: Instruction::Return
                            });
                            Ok(flow)
                        }
                    }
                    &None => {
                        if return_type == &Type::Void {
                            let mut flow = Flow::new();
                            flow.push_back(Node {
                                span: stmt.span.clone(),
                                instruction: Instruction::ReturnVoid
                            });
                            Ok(flow)

                        } else {
                            Err(Error::TypeError(stmt.span.clone()))
                        }
                    }
                }

            },
            &ast::StmtKind::Assign(ref assg) => {
                let (mut flow, t)  = Convert::convert_assg(assg, type_table)?;
                flow.push_back(Node {
                    span: stmt.span.clone(),
                    instruction: Instruction::Pop,
                });
                Ok(flow)
            },
            &ast::StmtKind::VarDelc(ref var_delc) => {
                Convert::convert_var_delc(var_delc, type_table)
            },
            &ast::StmtKind::If(ref check_expr, ref then_body, ref else_body) => {
                let (mut flow, t) = Convert::convert_expr(check_expr,
                                                          type_table)?;
                let mut then_flow = Convert::convert_stmt(then_body,
                                                          type_table,
                                                          return_type)?;

                match else_body {
                    &Some(ref body) => {
                        let mut else_flow = Convert::convert_stmt(body,
                                                                  type_table,
                                                                  return_type)?;
                        then_flow.push_back(Node {
                            span: stmt.span.clone(),
                            instruction: Instruction::Jump(
                                (else_flow.len() + 1) as i32)
                        });
                        flow.push_back(Node {
                            span: stmt.span.clone(),
                            instruction: Instruction::JumpIfZero(
                                (then_flow.len() + 1) as i32)
                        });
                        flow.append(&mut then_flow);
                        flow.append(&mut else_flow);
                        Ok(flow)
                    }
                    &None => {
                        flow.push_back(Node {
                            span: stmt.span.clone(),
                            instruction: Instruction::JumpIfZero(
                                (then_flow.len() + 1) as i32)
                        });
                        flow.append(&mut then_flow);
                        Ok(flow)

                    }
                }
            },
            &ast::StmtKind::Block(ref stmts) => {
                let mut flow = Convert::convert_stmts(stmts,
                                                 type_table,
                                                 return_type)?;
                flow.push_front(Node {
                    span: stmt.span.clone(),
                    instruction: Instruction::ScopeBegin
                });
                flow.push_back(Node {
                    span: stmt.span.clone(),
                    instruction: Instruction::ScopeEnd
                });
                Ok(flow)

            },
            _ => Err(Error::NotImplementedSyntax(stmt.span.clone()))
        }
    }
    
    pub fn convert_var_delc(var_delc: &ast::VarDelc,
                         type_table: &mut TypeTable) -> ConvertResult<Flow> {
        let mut flow = Flow::new();
        for &(ref t, ref id, ref expr) in &var_delc.names {
            let (mut delc_flow, t) = Convert::convert_single_var_delc(
                    t, id, type_table)?;
            flow.append(&mut delc_flow);
            match expr {
                &None => {},
                &Some(ref e) => {
                    let (mut assign_flow, _) = Convert::convert_simple_assign(
                        id, e, &e.span, type_table)?;
                }
            };
        }
        Ok(flow)

    }
    pub fn convert_single_var_delc(
            ast_type: &ast::Type,
            id: &ast::Id,
            type_table: &mut TypeTable) -> ConvertResult<(Flow, Type)> {
        if !type_table.can_declare(&id.node) {
            return Err(Error::AlreadyDeclaredVar(id.span.clone()));
        }
        let t = Type::from(&ast_type.node);
        let mut flow = Flow::new();
        flow.push_back(Node {
            span: id.span.clone(),
            instruction: Instruction::Declare(
                id.node.to_string(), t.clone())
        });
        type_table.insert(id.node.to_string(), t.clone());
        Ok((flow, t))

    }

    pub fn convert_simple_assign(id:&ast::Id,
                                 expr:&ast::Expr,
                                 span:&Span,
                                 type_table: &TypeTable
                                 ) -> ConvertResult<(Flow, Type)> {
        match type_table.get(&id.node) {
            Some(var_type) => {
                let (mut flow, t) = Convert::convert_expr(
                    expr, type_table)?;
                let mut cast_flow = Convert::auto_type_cast(
                    &t, var_type, span)?;
                flow.append(&mut cast_flow);
                flow.push_back(Node {
                    span: span.clone(),
                    instruction: Instruction::SaveVar(
                        id.node.to_string(),
                        var_type.clone())
                });

                Ok((flow, var_type.clone()))

            }
            None => Err(Error::NotDeclared(id.span.clone()))
        }

    }
    
    pub fn convert_assg(assg: &ast::Assg,
                        type_table: &TypeTable) -> ConvertResult<(Flow, Type)> {
        match &assg.node {
            &ast::AssgKind::Assign(ref id, ref expr) => {
                Convert::convert_simple_assign(id, expr, &assg.span, type_table)
            },
            &ast::AssgKind::Inc(ref id) => {
                match type_table.get(&id.node) {
                    Some(var_type) => {
                        let mut flow = Flow::new();
                        flow.push_back(Node {
                            span: assg.span.clone(),
                            instruction: Instruction::LoadVar(id.node.to_string())
                        });
                        flow.push_back(Node {
                            span: assg.span.clone(),
                            instruction: Instruction::LoadInt(1)
                        });

                        flow.push_back(Node {
                            span: assg.span.clone(),
                            instruction: Instruction::Addi
                        });

                        flow.push_back(Node {
                            span: assg.span.clone(),
                            instruction: Instruction::SaveVar(
                                id.node.to_string(),
                                var_type.clone())
                        });
                        Ok((flow, var_type.clone()))
                    },
                    None => Err(Error::NotDeclared(id.span.clone()))
                }
            },
            &ast::AssgKind::Dec(ref id) => {
                match type_table.get(&id.node) {
                    Some(var_type) => {
                        let mut flow = Flow::new();
                        flow.push_back(Node {
                            span: assg.span.clone(),
                            instruction: Instruction::LoadVar(id.node.to_string())
                        });
                        flow.push_back(Node {
                            span: assg.span.clone(),
                            instruction: Instruction::LoadInt(1)
                        });

                        flow.push_back(Node {
                            span: assg.span.clone(),
                            instruction: Instruction::Subi
                        });

                        flow.push_back(Node {
                            span: assg.span.clone(),
                            instruction: Instruction::SaveVar(
                                id.node.to_string(),
                                var_type.clone())
                        });
                        Ok((flow, var_type.clone()))
                    },
                    None => Err(Error::NotDeclared(id.span.clone()))
                }
            },
            &ast::AssgKind::PostInc(ref id) => {
                match type_table.get(&id.node) {
                    Some(var_type) => {
                        let mut flow = Flow::new();
                        flow.push_back(Node {
                            span: assg.span.clone(),
                            instruction: Instruction::LoadVar(id.node.to_string())
                        });
                        flow.push_back(Node {
                            span: assg.span.clone(),
                            instruction: Instruction::LoadVar(id.node.to_string())
                        });
                        flow.push_back(Node {
                            span: assg.span.clone(),
                            instruction: Instruction::LoadInt(1)
                        });
                        flow.push_back(Node {
                            span: assg.span.clone(),
                            instruction: Instruction::Addi
                        });
                        flow.push_back(Node {
                            span: assg.span.clone(),
                            instruction: Instruction::SaveVar(
                                id.node.to_string(),
                                var_type.clone())
                        });
                        flow.push_back(Node {
                            span: assg.span.clone(),
                            instruction: Instruction::Pop
                        });
                        Ok((flow, var_type.clone()))
                    },
                    None => Err(Error::NotDeclared(id.span.clone()))
                }
            },
            &ast::AssgKind::PostDec(ref id) => {
                match type_table.get(&id.node) {
                    Some(var_type) => {
                        let mut flow = Flow::new();
                        flow.push_back(Node {
                            span: assg.span.clone(),
                            instruction: Instruction::LoadVar(id.node.to_string())
                        });
                        flow.push_back(Node {
                            span: assg.span.clone(),
                            instruction: Instruction::LoadVar(id.node.to_string())
                        });
                        flow.push_back(Node {
                            span: assg.span.clone(),
                            instruction: Instruction::LoadInt(1)
                        });
                        flow.push_back(Node {
                            span: assg.span.clone(),
                            instruction: Instruction::Subi
                        });
                        flow.push_back(Node {
                            span: assg.span.clone(),
                            instruction: Instruction::SaveVar(
                                id.node.to_string(),
                                var_type.clone())
                        });
                        flow.push_back(Node {
                            span: assg.span.clone(),
                            instruction: Instruction::Pop
                        });
                        Ok((flow, var_type.clone()))
                    },
                    None => Err(Error::NotDeclared(id.span.clone()))
                }
            },
            _ => Err(Error::NotImplementedSyntax(assg.span.clone()))
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
                    &ast::LitKind::Float(ref f) => {
                        flow.push_back(Node::new(
                                lit.span.clone(),
                                Instruction::LoadFloat(*f)));
                        Ok((flow, Type::Float))
                    },
                }
            },
            &ast::ExprKind::Binary(ref bin_op, ref left, ref right) => {
                let (mut left_flow, left_type)  = Convert::convert_expr(
                    left, type_table)?;
                let (mut right_flow, right_type) = Convert::convert_expr(
                    right, type_table)?;
                let target_type = Convert::target_type_bin(&left_type,
                                                       &right_type,
                                                       &bin_op.node);
                left_flow.append(
                    &mut Convert::auto_type_cast(
                        &left_type, &target_type, &left.span)?);
                right_flow.append(
                    &mut Convert::auto_type_cast(
                        &right_type, &target_type, &right.span)?);

                left_flow.append(&mut right_flow);

                let (mut op, t) = Convert::convert_bin_op(bin_op, &target_type)?;
                left_flow.append(&mut op);

                Ok((left_flow, t))
            },
            &ast::ExprKind::Id(ref id) => {
                match type_table.get(&id.node) {
                    Some(t) => {
                        let mut flow = Flow::new();
                        flow.push_back(Node {
                            span: expr.span.clone(),
                            instruction: Instruction::LoadVar(id.node.to_string())
                        });
                        Ok((flow, t.clone()))
                        
                    },
                    None => Err(Error::NoVariable(expr.span.clone()))

                }
            },
            &ast::ExprKind::Call(ref id, ref exprs) => {
                Convert::convert_func_call(
                    id.node.to_string(), exprs, &expr.span, type_table)

            },
            &ast::ExprKind::Minus(ref operand) => {
                let (mut flow, t) = Convert::convert_expr(operand, type_table)?;
                let target = Convert::target_type_minus(&t);
                flow.append(&mut Convert::auto_type_cast(
                        &t,
                        &target,
                        &expr.span)?);
                let instruction = match target {
                    Type::Int => Ok(Instruction::Minusi),
                    Type::Float => Ok(Instruction::Minusf),
                    _ => Err(Error::TypeError(expr.span.clone())),
                }?;

                flow.push_back(Node {
                    span: expr.span.clone(),
                    instruction
                });
                Ok((flow, target))
            },
            &ast::ExprKind::Assign(ref assg) => {
                Convert::convert_assg(assg, type_table)
            },
            &ast::ExprKind::Not(ref operand) => {
                let (mut flow, t) = Convert::convert_expr(operand, type_table)?;
                flow.append(&mut Convert::auto_type_cast(
                        &t,
                        &Type::Int,
                        &expr.span)?);

                flow.push_back(Node {
                    span: expr.span.clone(),
                    instruction: Instruction::Not
                });
                Ok((flow, Type::Int))

            }
            _ => {
                Err(Error::NotImplementedSyntax(expr.span.clone()))
            }
        }
    }

    pub fn convert_bin_op(op: &ast::BinOp, t: &Type) -> ConvertResult<(Flow, Type)> {
        let mut result = Flow::new();
        match &op.node {
            &ast::BinOpKind::Add => match t {
                &Type::Int => {
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Addi,
                    });
                    Ok((result, Type::Int))
                },
                &Type::Float => {
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Addf,
                    });
                    Ok((result, Type::Float))
                },
                _ => Err(Error::TypeError(op.span.clone()))
            },
            &ast::BinOpKind::Sub => match t {
                &Type::Int => {
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Subi,
                    });
                    Ok((result, Type::Int))
                },
                &Type::Float => {
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Subf,
                    });
                    Ok((result, Type::Float))
                },
                _ => Err(Error::TypeError(op.span.clone()))
            },
            &ast::BinOpKind::Mul => match t {
                &Type::Int => {
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Muli,
                    });
                    Ok((result, Type::Int))
                },
                &Type::Float => {
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Mulf,
                    });
                    Ok((result, Type::Float))
                },
                _ => Err(Error::TypeError(op.span.clone()))
            },
            &ast::BinOpKind::Div => match t {
                &Type::Int => {
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Divi,
                    });
                    Ok((result, Type::Int))
                },
                &Type::Float => {
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Divf,
                    });
                    Ok((result, Type::Float))
                },
                _ => Err(Error::TypeError(op.span.clone()))
            },
            &ast::BinOpKind::And => match t {
                &Type::Int => {
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::And,
                    });
                    Ok((result, Type::Int))
                },
                _ => Err(Error::TypeError(op.span.clone()))
            },
            &ast::BinOpKind::Or => match t {
                &Type::Int => {
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Or,
                    });
                    Ok((result, Type::Int))
                },
                _ => Err(Error::TypeError(op.span.clone()))
            },
            &ast::BinOpKind::Eq => match t {
                &Type::Int => {
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Eqi,
                    });
                    Ok((result, Type::Int))
                },
                &Type::Float => {
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Eqf,
                    });
                    Ok((result, Type::Int))
                },
                _ => Err(Error::TypeError(op.span.clone()))
            },
            &ast::BinOpKind::Ne => match t {
                &Type::Int => {
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Eqi,
                    });
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Not,
                    });
                    Ok((result, Type::Int))
                },
                &Type::Float => {
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Addf,
                    });
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Not,
                    });
                    Ok((result, Type::Int))
                },
                _ => Err(Error::TypeError(op.span.clone()))
            },
            &ast::BinOpKind::Lt => match t {
                &Type::Int => {
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Lti,
                    });
                    Ok((result, Type::Int))
                },
                &Type::Float => {
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Ltf,
                    });
                    Ok((result, Type::Int))
                },
                _ => Err(Error::TypeError(op.span.clone()))
            },
            &ast::BinOpKind::Le => match t {
                &Type::Int => {
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Gti,
                    });

                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Not,
                    });

                    Ok((result, Type::Int))
                },
                &Type::Float => {
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Gtf,
                    });

                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Not,
                    });

                    Ok((result, Type::Int))
                },
                _ => Err(Error::TypeError(op.span.clone()))
            },
            &ast::BinOpKind::Gt => match t {
                &Type::Int => {
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Gti,
                    });
                    Ok((result, Type::Int))
                },
                &Type::Float => {
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Gtf,
                    });

                    Ok((result, Type::Int))
                },
                _ => Err(Error::TypeError(op.span.clone()))
            },
            &ast::BinOpKind::Ge => match t {
                &Type::Int => {
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Lti,
                    });

                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Not,
                    });

                    Ok((result, Type::Int))
                },
                &Type::Float => {
                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Ltf,
                    });

                    result.push_back(Node {
                        span: op.span.clone(),
                        instruction: Instruction::Not,
                    });

                    Ok((result, Type::Int))
                },
                _ => Err(Error::TypeError(op.span.clone()))
            },
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
                (&Type::Char, &Type::Float) => {
                    result.push_back(Node {
                        span: span.clone(),
                        instruction: Instruction::CharToInt
                    });
                    result.push_back(Node {
                        span: span.clone(),
                        instruction: Instruction::IntToFloat
                    });
                },
                (&Type::Int, &Type::Float) => {
                    result.push_back(Node {
                        span: span.clone(),
                        instruction: Instruction::IntToFloat
                    });
                },
                (&Type::Float, &Type::Int) => {
                    result.push_back(Node {
                        span: span.clone(),
                        instruction: Instruction::FloatToInt
                    });

                }
                (_, _) => return Err(Error::TypeError(span.clone())),
            };
            Ok(result)
        }
    }

    pub fn target_type_minus(t:&Type) -> Type {
        match t {
            &Type::Int => Type::Int,
            &Type::Float => Type::Float,
            _ => Type::Int,
        }
    }

    pub fn target_type_bin<'a>(
            t1:&'a Type, t2:&'a Type, op: &ast::BinOpKind) -> &'a Type {
        match op {
            &ast::BinOpKind::Add | &ast::BinOpKind::Sub |
                    &ast::BinOpKind::Mul | &ast::BinOpKind::Div |
                    &ast::BinOpKind::Eq | &ast::BinOpKind::Ne |
                    &ast::BinOpKind::Lt | &ast::BinOpKind::Le |
                    &ast::BinOpKind::Gt | &ast::BinOpKind::Ge => {

                let l1 = Convert::type_level(t1);
                let l2 = Convert::type_level(t2);
                let li = Convert::type_level(&Type::Int);

                if l1 < li && l2 < li {
                    &Type::Int
                } else if l1 > l2 {
                    t1
                } else {
                    t2
                }
            },
            &ast::BinOpKind::And | &ast::BinOpKind::Or => {
                &Type::Int
            }
        }
    }
    
    pub fn type_level(t:&Type) -> u32{
        match t {
            &Type::Void => 0,
            &Type::Pointer(_) => 0,
            &Type::Arrow(_, _) => 0,
            &Type::Char => 1,
            &Type::Int => 2,
            &Type::Float => 3
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
fn hello_world() {
    let code = r#"int main(void) {
        printf("Hello Wordl!");
    }"#;
    let meta = MetaData::new(code.to_string());
    let ast = parse(&meta).unwrap();
    let func_table = Convert::convert_program(&ast).unwrap();
    assert_eq!(func_table.len(), 1);
}

#[test]
fn simple_calc() {
let code = r#"
int add(int a, int b) {
    return a + b;
}
int main(void) {
    printf("%d", add(1, 2));
}"#;
    let meta = MetaData::new(code.to_string());
    let ast = parse(&meta).unwrap();
    let func_table = Convert::convert_program(&ast).unwrap();
    assert_eq!(func_table.len(), 2);
}
