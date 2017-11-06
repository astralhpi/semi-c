pub mod semic;
pub mod ast;

#[test]
fn charcon() {
    assert_eq!(semic::parse_Charcon("\'a\'").unwrap(), 'a');
    assert_eq!(semic::parse_Charcon("\'Z\'").unwrap(), 'Z');
    assert_eq!(semic::parse_Charcon("\'\\n\'").unwrap(), '\n');
    assert_eq!(semic::parse_Charcon("\'\\0\'").unwrap(), '\0');
}

#[test]
fn intcon() {
    assert_eq!(semic::parse_Intcon("123").unwrap(), 123);
    assert_eq!(semic::parse_Intcon("323").unwrap(), 323);
}

#[test]
fn stringcon() {
    assert_eq!(semic::parse_Stringcon("\"\"").unwrap(), "");
    assert_eq!(semic::parse_Stringcon("\"hello\"").unwrap(), "hello");
    assert_eq!(
        semic::parse_Stringcon("\"hello\nworld\"").unwrap(),
        "hello\nworld");
}
fn comment() {
    assert_eq!(semic::parse_Comments("/**/").unwrap(), "/**/");
    assert_eq!(semic::parse_Comments("/*asf\n*/").unwrap(), "/*asf\n*/");
}

fn main() {
    match semic::parse_Id("  \nasdf") {
        Ok(data) => print!("{:?}", data),
        Err(err) => print!("{:?}", err),
    }
}
