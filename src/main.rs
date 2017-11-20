extern crate parser;
extern crate lalrpop_util;

use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

use lalrpop_util::ParseError::*;

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

fn parse(code: &str) -> Result<parser::ast::Program, String> {
    let line_breaks_pos = get_line_break_positions(&code);
    match parser::semic::parse_Prog(&code) {
        Ok(ast) => Result::Ok(ast),
        Err(e) => match e {
            UnrecognizedToken {token, expected} => {
                let _expected = expected;
                match token {
                    Option::None => Result::Err(
                        format!(
                            "Syntax error : line {}",
                            line_breaks_pos.len())),
                    Option::Some((l, _, _)) => {
                        let (line, _) = line_column(&line_breaks_pos, l);
                        Result::Err(format!(
                                "Syntax error: line {}", line + 1))
                    }
                }
            },
            InvalidToken {location} => {
                let (line, _) = line_column(&line_breaks_pos, location);
                Result::Err(format!("Syntax error: line {}", line + 1))
            },
            ExtraToken {token} => {
                let (left, _, _) = token;
                let (line, _) = line_column(&line_breaks_pos, left);
                Result::Err(format!("Syntax error: line {}", line + 1))
            },
            _ => panic!("wtf")
        }
    }
}

#[test]
fn test_parse() {
    let code = r#"int avg(int count, int *value) {
    int i, total
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
    match parse(code) {
        Ok(_) => {
            assert!(false);
        }
        Err(e) => {
            assert_eq!(e, "Syntax error: line 3");

        }

    }

}


fn main() {
    let code = read("input.c");

    let ast = parse(&code);
}
