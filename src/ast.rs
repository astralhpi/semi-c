#[derive(Debug, PartialEq)]
pub struct Span {
    pub lo: usize,
    pub hi: usize,
}

impl Span {
    pub fn new(lo:usize, hi:usize) -> Span {
        Span {
            lo,
            hi
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Spanned<T> {
    pub span: Span,
    pub node: T,
}

impl<T> Spanned<T> {
    pub fn new(lo:usize, hi:usize, node:T) -> Spanned<T> {
        Spanned {
            span: Span {
                lo,
                hi
            },
            node
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum BinOpKind {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

pub type BinOp = Spanned<BinOpKind>;

#[derive(Debug, PartialEq)]
pub enum LitKind {
    Int(i32),
    Char(char),
    String(String),
}

pub type Lit = Spanned<LitKind>;

pub type Id = Spanned<String>;

#[derive(Debug, PartialEq)]
pub struct Expr {
    pub span: Span,
    pub node: ExprKind,
}

#[derive(Debug, PartialEq)]
pub enum ExprKind {
    Minus(Box<Expr>),
    Not(Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Call(Id, Vec<Expr>),
    Index(Id, Box<Expr>),
    Paren(Box<Expr>),
    Lit(Box<Lit>)
}

#[derive(Debug, PartialEq)]
pub struct Assg {
    pub span: Span,
    pub node: AssgKind,
}

#[derive(Debug, PartialEq)]
pub enum AssgKind {
    // id[expr] = expr
    Assign(Id, Option<Box<Expr>>, Box<Expr>)
}

#[derive(Debug, PartialEq)]
pub struct Stmt {
    pub span: Span,
    pub node: StmtKind,
}

#[derive(Debug, PartialEq)]
pub enum StmtKind {
    // if(expr) stmt else stmt
    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>),
    // while(expr) stmt
    While(Box<Expr>, Box<Stmt>),
    // for(assg;expr;assg) stmt
    For(Box<Assg>, Box<Expr>, Box<Assg>, Box<Stmt>),
    // return expr
    Return(Option<Box<Expr>>),
    Assign(Box<Assg>),
    Call(Id, Vec<Expr>),
    Block(Box<Stmt>),
    Empty,
}


#[derive(Debug, PartialEq)]
pub enum ParamTypesKind {
    Void,
    Params(Box<Vec<(Type, Id)>>)
}

type ParamTypes = Spanned<ParamTypesKind>;

#[derive(Debug, PartialEq)]
pub enum TypeKind {
    Char,
    Int,
}

type Type = Spanned<TypeKind>;

#[derive(Debug, PartialEq)]
pub enum ReturnTypeKind {
    Void,
    Type(TypeKind)
}

type ReturnType = Spanned<ReturnTypeKind>;

#[derive(Debug, PartialEq)]
pub struct Program {
    pub span: Span,
    pub declines: Vec<GlobalDclKind>
}

#[derive(Debug, PartialEq)]
pub enum GlobalDclKind {
    Var(Type, VarDelc),
    Func(FuncDelc, FuncBody),
    ExternFunc(FuncDelc)
}

#[derive(Debug, PartialEq)]
pub struct VarDelc {
    pub span: Span,
    pub var_type: Type,
    pub names: Vec<VarName>
}

#[derive(Debug, PartialEq)]
pub enum VarNameKind {
    Single(Id),
    Array(Id, u32),
}
type VarName = Spanned<VarNameKind>;

#[derive(Debug, PartialEq)]
pub struct FuncDelc {
    pub span: Span,
    pub return_type: ReturnType,
    pub id: Id,
    pub param_types: ParamTypes,
}

#[derive(Debug, PartialEq)]
pub struct FuncBody {
    pub span: Span,
    pub var_decls: Vec<VarDelc>,
    pub stmt: Stmt,
}


