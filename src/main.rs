extern crate parser;
extern crate lalrpop_util;
#[macro_use]
extern crate indoc;

mod meta_data;
mod error;
mod symbol_table;
mod flow_graph;
mod helper;
mod runtime;
mod memory;
mod register;
mod printf;

use meta_data::MetaData;
use helper::{read, parse};

fn main() {
    let code = read("input.c");
    let meta = MetaData::new(code);
    parse(&meta);
}
