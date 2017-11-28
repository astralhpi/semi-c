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
        semic::parse_Lit("\"hello\\nworld\"").unwrap(),
        create_lit_string(0, 14, "hello\nworld".to_string()));
}


#[test]
fn id() {
    assert_eq!(
        semic::parse_Id("asdf").unwrap().node,
        "asdf")
}

#[test]
fn expr() {
    assert_eq!(
        semic::parse_Expr("(123)").unwrap().node,
        ExprKind::Paren(Box::new(Expr {
            span: Span::new(1, 4),
            node: ExprKind::Lit(Box::new(Lit::new(1, 4, LitKind::Int(123))))
        })));
    match semic::parse_Expr("1+2*3 == 3").unwrap().node {
        ExprKind::Binary(op, _left, _right) => {
            assert_eq!(op.node, BinOpKind::Eq);
        },
        _ => assert!(false)
    };
    match semic::parse_Expr("a").unwrap().node {
        ExprKind::Id(id) => {
            assert_eq!(id.node, "a");
        },
        _ => assert!(false)
    };

    match semic::parse_Expr("a*b == c").unwrap().node {
        ExprKind::Binary(op, _left, _right) => {
            assert_eq!(op.node, BinOpKind::Eq);
        },
        _ => assert!(false)
    };

    match semic::parse_Expr("a[1+2]").unwrap().node {
        ExprKind::Index(id, _expr) => {
            assert_eq!(id.node, "a");
        },
        _ => assert!(false)
    };

    match semic::parse_Expr("a(1, 2)").unwrap().node {
        ExprKind::Call(id, _args) => {
            assert_eq!(id.node, "a");
        },
        _ => assert!(false)
    };

    match semic::parse_Expr("a()").unwrap().node {
        ExprKind::Call(id, _args) => {
            assert_eq!(id.node, "a");
        },
        _ => assert!(false)
    };
}

#[test]
fn program() {
    let code = r#"
int avg(int count, int *value) {
    int i, total;
    int sum = 0;
    for (i = 1; i < count; i++) {
        int a;
        total = total + value[i];
    }

    return (total / count);
}

int main(void) {
    int studentNumber, count, i, sum;
    int mark[4];
    float average;
    
    count = 4;
    sum = 0;

    for (i=0; i < count; i++) {
        mark[i] = i * 30;
        sum = sum + mark[i];
        average = avg(i + 1, mark);
        if (average > 40) {
            printf("%f\n", average);
        }
    }
}
"#;
    let ast = semic::parse_Prog(code);
    match ast {
        Ok(e) => print!("{:?}", e),
        Err(e) => {
            print!("{:?}", e);
            assert!(false);
        }

    }
}
