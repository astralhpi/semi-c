#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
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
    Id(Id),
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
    Assign(Id, Option<Box<Expr>>, Box<Expr>),
    Inc(Id),
    Dec(Id)
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
    For(Option<Box<Assg>>, Option<Box<Expr>>, Option<Box<Assg>>, Box<Stmt>),
    // return expr;
    Return(Option<Box<Expr>>),
    Assign(Box<Assg>),
    Call(Id, Vec<Expr>),
    Block(Vec<Stmt>),
    VarDelc(Box<VarDelc>),
    Empty,
}


#[derive(Debug, PartialEq)]
pub enum ParamTypesKind {
    Void,
    Params(Vec<(Type, Id)>)
}

pub type ParamTypes = Spanned<ParamTypesKind>;

#[derive(Debug, PartialEq, Clone)]
pub enum TypeKind {
    Char,
    Int,
    Float,
    Pointer(Box<TypeKind>),
    Array(Box<TypeKind>, Option<i32>)
}

pub type Type = Spanned<TypeKind>;

#[derive(Debug, PartialEq)]
pub enum ReturnTypeKind {
    Void,
    Type(TypeKind)
}

pub type ReturnType = Spanned<ReturnTypeKind>;

#[derive(Debug, PartialEq)]
pub struct Program {
    pub span: Span,
    pub declines: Vec<GlobalDclKind>
}

#[derive(Debug, PartialEq)]
pub enum GlobalDclKind {
    Var(VarDelc),
    Func(FuncDelc, FuncBody),
}


#[derive(Debug, PartialEq)]
pub struct VarDelc {
    pub span: Span,
    pub names: Vec<(Type, Id, Option<Expr>)>
}

#[derive(Debug, PartialEq)]
pub enum VarNameKind {
    Single(Id, Option<Expr>),
    Array(Id, i32),
}
pub type VarName = Spanned<VarNameKind>;

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
    pub stmt: Vec<Stmt>,
}


