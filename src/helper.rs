extern crate parser;
extern crate lalrpop_util;

use meta_data::MetaData;
use error::syntax_error;
use lalrpop_util::ParseError::*;
use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

pub fn parse(meta: &MetaData) -> Result<parser::ast::Program, String> {
    match parser::semic::parse_Prog(&meta.code) {
        Ok(ast) => Result::Ok(ast),
        Err(e) => {
            print!("{:?}", e);

            match e {
                UnrecognizedToken {token, expected} => {
                    let _expected = expected;
                    match token {
                        Option::None => Result::Err(
                            format!(
                                "Syntax error : line {}",
                                meta.line_count())),
                        Option::Some((l, _, _)) => {
                            syntax_error(meta, l)
                        }
                    }
                },
                InvalidToken {location} => {
                    syntax_error(meta, location)
                },
                ExtraToken {token} => {
                    let (left, _, _) = token;
                    let (line, _) = meta.line_column(left);
                    syntax_error(meta, line)
                },
                _ => panic!("wtf")
            }
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
    let meta = MetaData::new(code.to_string());
    match parse(&meta) {
        Ok(_) => {
            assert!(false);
        }
        Err(e) => {
            assert_eq!(e, "Syntax error: line 3");

        }

    }
}

pub fn read(path:&str) -> String {
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

