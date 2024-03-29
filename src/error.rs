use meta_data::MetaData;
use flow_graph::Type;
use parser::ast::Span;

#[derive(PartialEq, Clone, Debug)]
pub enum Error {
    TypeError(Span),
    NotImplementedSyntax(Span),
    NotDeclared(Span),
    NoVariable(Span),
    Runtime(String),
    NoMain,
    NoScope,
    AlreadyDeclaredVar(Span),
    NotImplementedRuntime(String, Span),
    NoArraySize(Span),
    DivideByZero(Span),
}

pub fn syntax_error<T>(meta:&MetaData, loc:usize) -> Result<T, String> {
    let (line, _) = meta.line_column(loc);
    Result::Err(format!("Syntax error : line {}", line + 1))
}
