pub struct Span {
    pub lo: usize,
    pub hi: usize,
}

pub struct Spanned<T> {
    pub span: Span,
    pub node: T,
}

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

pub enum LitKind {
    Int(i32),
    Char(char),
    String(String),
}

pub type Lit = Spanned<LitKind>;

pub type Id = Spanned<String>;

pub struct Expr {
    span: Span,
    node: ExprKind,
}

pub enum ExprKind {
    Minus(Box<Expr>),
    Not(Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Call(Id, Vec<Expr>),
    Index(Id, Box<Expr>),
    Paren(Box<Expr>),
    Lit(Box<Lit>)
}

pub struct Assg {
    span: Span,
    node: AssgKind,
}

pub enum AssgKind {
    // id[expr] = expr
    Assign(Id, Option<Box<Expr>>, Box<Expr>)
}

pub struct Stmt {
    span: Span,
    node: StmtKind,
}

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


pub enum ParamTypesKind {
    Void,
    Params(Box<Vec<(Type, Id)>>)
}

type ParamTypes = Spanned<ParamTypesKind>;

pub enum TypeKind {
    Char,
    Int,
}

type Type = Spanned<TypeKind>;

pub enum ReturnTypeKind {
    Void,
    Type(TypeKind)
}

type ReturnType = Spanned<ReturnTypeKind>;

pub struct Program {
    span: Span,
    declines: Vec<GlobalDclKind>
}

pub enum GlobalDclKind {
    Var(Type, VarDelc),
    Func(FuncDelc, FuncBody),
    ExternFunc(FuncDelc)
}

pub struct VarDelc {
    span: Span,
    var_type: Type,
    names: Vec<VarName>
}

pub enum VarNameKind {
    Single(Id),
    Array(Id, u32),
}
type VarName = Spanned<VarNameKind>;

pub struct FuncDelc {
    span: Span,
    return_type: ReturnType,
    id: Id,
    param_types: ParamTypes,
}

pub struct FuncBody {
    span: Span,
    var_decls: Vec<VarDelc>,
    stmt: Stmt,
}


