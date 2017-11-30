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
use runtime::{Runtime, ProgramState};
use parser::ast;
use std::io;

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
    let mut runtime = Runtime::new(meta, program);
    runtime.prepare();

    loop {
        let cmd = Command::read_command();
        if cmd.is_none() { continue; }

        match cmd.unwrap() {
            Command::Next(n) => match runtime.step_line(n) {
                Err(err) => {
                    show_error(err, &runtime.meta);
                    process::exit(1);
                },
                Ok(state) => match state {
                    ProgramState::End => {
                        println!("End of Program");
                    }
                    _ => {}
                }
            },
            Command::Print(x) => match runtime.print(&x) {
                None => println!("Invisible variable"),
                Some(s) => println!("{}",s)
            },
            Command::Trace(x) => match runtime.trace(&x) {
                None => println!("Invisible variable"),
                Some(history) => {
                    print_trace(history, &x);

                }
            }
        }

    }
}

fn print_trace(history:&Vec<(usize, Option<String>)>, name:&str) {
    for &(ref line, ref val) in history {
        print!("{} = ", name);
        match val {
            &None => print!("N/A"),
            &Some(ref s) => print!("{}", s)
        };
        println!(" at line {}", line + 1);
    }
}

pub enum Command {
    Next(u32),
    Print(String),
    Trace(String),
}

impl Command {
    fn read_command() -> Option<Command> {
        print!(">> ");
        io::stdout().flush();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                let input = input.replace("\n", "");
                let sp: Vec<_> = input.split(" ").collect();
                match sp[0] {
                    "next" => match Command::parse_next(&sp) {
                        Ok(c) => Some(c),
                        Err(e) => {
                            println!("{}", e);
                            None
                        }
                    },
                    "trace" => match Command::parse_trace(&sp) {
                        Ok(c) => Some(c),
                        Err(e) => {
                            println!("{}", e);
                            None
                        }
                    },
                    "print" => match Command::parse_print(&sp) {
                        Ok(c) => Some(c),
                        Err(e) => {
                            println!("{}", e);
                            None
                        }
                    },
                    _ => {
                        println!("Available commands : next, print, trace");
                        None

                    }
                }
            },
            _ => None
        }
    }

    fn parse_print(sp: &Vec<&str>) -> Result<Command, &'static str> {
        let err = Err("Invalid typing of the variable name") ;
        if sp.len() != 2 {
            err
        } else {
            match parser::semic::parse_Id(sp[1]) {
                Ok(_) => Ok(Command::Print(sp[1].to_string())),
                Err(_) => err
            }
        }

    }

    fn parse_trace(sp: &Vec<&str>) -> Result<Command, &'static str> {
        let err = Err("Invalid typing of the variable name") ;
        if sp.len() != 2 {
            err
        } else {
            match parser::semic::parse_Id(sp[1]) {
                Ok(_) => Ok(Command::Trace(sp[1].to_string())),
                Err(_) => err
            }
        }

    }

    fn parse_next(sp: &Vec<&str>) -> Result<Command, &'static str> {
        let err = Err("Incorrect command usage : try 'next [lines]'") ;
        if sp.len() < 2 {
            Ok(Command::Next(1))
        } else if sp.len() > 2 {
            err
        } else {
            match sp[1].parse() {
                Ok(i) => {
                    Ok(Command::Next(i))
                },
                Err(e) =>  {
                    err
                }
            }
        }

    }

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
