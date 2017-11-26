use meta_data::MetaData;
use flow_graph::Type;
use parser::ast::Span;

pub enum Error {
    TypeError(Span),

}

pub fn syntax_error<T>(meta:&MetaData, loc:usize) -> Result<T, String> {
    let (line, _) = meta.line_column(loc);
    Result::Err(format!("Syntax error: line {}", line + 1))
}
