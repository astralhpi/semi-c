#![allow(unused)]
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
use std::path::Path;
use std::io::prelude::*;
use std::fs::File;
use std::error::Error;
use std::collections::HashMap;

use meta_data::MetaData;
use flow_graph::{Convert, Func};
use runtime::Runtime;
use parser::ast;

fn read(path:&str) -> String {
    let path = Path::new(path);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => {
            print!("couldn't open {}: {}", display, why.description());
            process::exit(1);
        }
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => {
            print!("couldn't read {}: {}", display, why.description());
            process::exit(1);
        }
        Ok(_)  => s
    }

}

fn parse(meta: &MetaData) -> ast::Program {
    match helper::parse(&meta) {
        Ok(ast) => ast,
        Err(err) => {
            println!("{}", err);
            process::exit(1);
        }
    }
}

fn compile(ast: ast::Program, meta: &MetaData) -> HashMap<String, Func> {
    match Convert::convert_program(&ast) {
        Ok(p) => {
            p
        },
        Err(e) => {
            show_error(e, meta);
            process::exit(1);
        }
    }
}

fn show_error(e: error::Error, meta: &MetaData) {
    match e {
        error::Error::TypeError(span) => {
            println!("Type error : line {}", meta.line(span.lo) + 1);
        },
        error::Error::NotImplementedSyntax(span) => {
            println!("Syntax error : line {}", meta.line(span.lo) + 1);

        },
        error::Error::NotDeclared(span) => {
            println!("Not declared variable error : line {}", meta.line(span.lo) + 1);

        },
        error::Error::NoVariable(span) => {
            println!("Not declared variable error : line {}", meta.line(span.lo) + 1);
        },
        error::Error::Runtime(s) => {
            println!("Run-time error : {}", s);
        },
        error::Error::NoMain => {
            println!("Run-time error : No Main Function");
        },
        error::Error::NoScope => {
            println!("Run-time error : No Scope");
        },
        error::Error::AlreadyDeclaredVar(span) => {
            println!("Duplicated variable error : line {}", meta.line(span.lo) + 1);

        },
        error::Error::NotImplementedRuntime(s, span) => {
            println!("Run-time error : {}", meta.line(span.lo));
        },
        error::Error::NoArraySize(span) => {
            println!("Type error : {}", meta.line(span.lo));
        }
    }

}
fn run(filename:&str) {
    let code = read(filename);
    let meta = MetaData::new(code);
    let ast = parse(&meta);
    let program = compile(ast, &meta);
}

fn main() {
    let mut args = env::args();
    let path = args.next().unwrap();

    match args.next() {
        None => println!("[Usage]\n{} <c_file>", path),
        Some(filename) => {
            run(&filename);
        }
    }
}
