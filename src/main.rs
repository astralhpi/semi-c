extern crate parser;

use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

fn read(path:&str) -> String {
    let path = Path::new(path);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why.description()),
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, why.description()),
        Ok(_)  => s
    }

}

fn get_line_break_positions(code: &str) -> Vec<usize> {
    let mut line_breaks_pos: Vec<usize> = vec![];
    let mut i = 0;
    for c in code.chars() {
        if c == '\n' {
            line_breaks_pos.push(i);
        }
        i += 1;
    }
    line_breaks_pos 
}

fn line_column(line_break_pos: &Vec<usize>, pos: usize) -> (usize, usize) {
    let mut line = 0;
    for i in line_break_pos {
        if pos < i + 1 {
            break;
        }
        line += 1;
    }

    if line == 0 {
        (0, pos)

    }
    else {
        (line, pos - line_break_pos[line - 1] - 1)
    }
}

#[test]
fn test_line_column() {
    let code = "a\nbc\ndef";
    let line_breaks_pos = get_line_break_positions(&code);
    assert_eq!(vec![1, 4], line_breaks_pos);
    assert_eq!((0,0), line_column(&line_breaks_pos, 0));
    assert_eq!((0,1), line_column(&line_breaks_pos, 1));
    assert_eq!((1,0), line_column(&line_breaks_pos, 2));
    assert_eq!((1,1), line_column(&line_breaks_pos, 3));
    assert_eq!((1,2), line_column(&line_breaks_pos, 4));
    assert_eq!((2,0), line_column(&line_breaks_pos, 5));
    assert_eq!((2,1), line_column(&line_breaks_pos, 6));
}

fn main() {
    let code = read("input.c");

    let line_breaks_pos = get_line_break_positions(&code);
    print!("{:?}\n", &code[102..]);
    print!("{:?}\n", line_column(&line_breaks_pos, 102));
}
