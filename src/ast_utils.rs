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
