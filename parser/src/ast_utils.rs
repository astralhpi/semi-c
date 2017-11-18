use ast::*;

pub fn create_lit_char(l:usize, r:usize, c:char) -> Lit {
    Lit::new(l, r, LitKind::Char(c))
}

pub fn create_lit_string(l:usize, r:usize, s:String) -> Lit {
    Lit::new(l, r, LitKind::String(s))
}

pub fn create_lit_int(l:usize, r:usize, i:i32) -> Lit {
    Lit::new(l, r, LitKind::Int(i))
}

pub fn create_expr_binary(
        l:usize, r:usize, l2:usize, r2:usize,
        bin_op:BinOpKind, left:Expr, right:Expr) -> Expr {
    Expr {
        span: Span::new(l, r),
        node: ExprKind::Binary(
            BinOp::new(l2, r2, bin_op),
            Box::new(left), Box::new(right))
    }
}

pub fn create_expr_id(l:usize, r:usize, id:Id) -> Expr {
    Expr {
        span: Span::new(l, r),
        node: ExprKind::Id(id)
    }
}
