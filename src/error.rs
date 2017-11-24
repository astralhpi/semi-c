use meta_data::MetaData;

pub fn syntax_error(meta:&MetaData, loc:usize) -> String {
    let (line, _) = meta.line_column(loc);
    format!("Syntax error: line {}", line + 1)
}
