pub mod semic;
pub mod ast;
pub mod ast_utils;

use ast::*;
use ast_utils::*;

#[test]
fn lit() {
    assert_eq!(
        semic::parse_Lit("\'a\'").unwrap().node,
        LitKind::Char('a'));
    assert_eq!(
        semic::parse_Lit("\'Z\'").unwrap().node,
        LitKind::Char('Z'));
    assert_eq!(
        semic::parse_Lit("\'\\n\'").unwrap().node,
        LitKind::Char('\n'));
    assert_eq!(
        semic::parse_Lit("\'\\0\'").unwrap().node,
        LitKind::Char('\0'));
    assert_eq!(
        semic::parse_Lit("123").unwrap(), 
        create_lit_int(0, 3, 123));
    assert_eq!(
        semic::parse_Lit("323").unwrap(),
        create_lit_int(0, 3, 323));
    assert_eq!(
        semic::parse_Lit("\"\"").unwrap(),
        create_lit_string(0, 2,"".to_string()));
    assert_eq!(
        semic::parse_Lit("\"hello\"").unwrap(),
        create_lit_string(0, 7, "hello".to_string()));
    assert_eq!(
        semic::parse_Lit("\"hello\nworld\"").unwrap(),
        create_lit_string(0, 13, "hello\nworld".to_string()));
}


#[test]
fn id() {
    assert_eq!(
        semic::parse_Id("asdf").unwrap().node,
        "asdf")
}

//#[test]
//fn basic_op() {
//    assert_eq!(semic::parse_Expr("10"), ast::Expr {
//        span: ast::Span {
//            lo: 0,
//            hi: 1,
//        },
//        node: ast::ExprKind::Lit(Box::new(ast::Spanned {
//            span: ast::Span {
//                lo: 0,
//                hi: 1,
//            },
//            node: ast::LitKind::Int(10)
//        }
//    ))});
//
//}

fn main() {
    match semic::parse_Id("  \nasdf") {
        Ok(data) => print!("{:?}", data),
        Err(err) => print!("{:?}", err),
    }
}
