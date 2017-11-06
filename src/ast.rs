pub struct Span {
    pub lo: usize,
    pub hi: usize,
}

pub struct Spanned<T> {
    pub span: Span,
    pub item: T,
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

pub type BindOp = Spanned<BinOpKind>;


pub struct Expr {
    span: Span,
    node: ExprKind,

}

pub enum ExprKind {
    Minus(Box<Expr>),
    Not(Box<Expr>),
}
