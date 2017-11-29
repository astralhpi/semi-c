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

use std::{process, env};
use meta_data::MetaData;
use helper::{read, parse};
use flow_graph::Convert;
use runtime::Runtime;

fn main() {
    let mut args = env::args();
    let path = args.next().unwrap();

    match args.next() {
        None => println!("[Usage]\n{} <c_file>", path),
        Some(filename) => {
            let code = read(&filename);
            let meta = MetaData::new(code);
            let ast = parse(&meta).unwrap();
            match Convert::convert_program(&ast) {
                Ok(p) => {
                    let mut runtime = Runtime::new(meta, p);
                    runtime.run();
                },
                Err(e) => {
                    process::exit(1);

                }
            }

        }
    }
}
