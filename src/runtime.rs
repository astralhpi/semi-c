use std::collections::{HashMap};
use flow_graph::{Func, Convert, Instruction, Node};
use helper::parse;
use meta_data::MetaData;
use symbol_table::SymbolTable;
use error::Error;


pub struct Runtime {
    flow_table: HashMap<String, Func>,
    register_stack: Vec<Register>,
    memory: Memory,
    program_stack: Vec<(String, usize)>,
}

pub struct Memory {
    memory: Vec<u8>,
    stack_pointer: Vec<usize>
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            memory: Vec::new(),
            stack_pointer: Vec::new()
        }
    }
    pub fn push_scope(&mut self) {
        let len = self.stack_pointer.len();
        let last = if len == 0 {
            0
        } else {
            self.stack_pointer[len - 1]
        };
        self.stack_pointer.push(last);
    }

    pub fn drop_scope(&mut self) {
        self.stack_pointer.pop();
    }

    pub fn alloc_stack(&mut self, size: usize) -> Result<usize, Error> {
        let address = self.stack_pointer.pop().ok_or(Error::Runtime)?;
        self.stack_pointer.push(address + size);
        Ok(address)
    }
}

#[derive(Debug)]
pub enum ProgramState {
    Processing,
    End,
}


impl Runtime {
    fn new(flow_table: HashMap<String, Func>) -> Runtime {
        Runtime {
            flow_table,
            register_stack: Vec::new(),
            memory: Memory::new(),
            program_stack: Vec::new()
        }
    }
    fn run(&mut self) -> Result<(), Error> {
        self.flow_table.get("main").ok_or(Error::NoMain)?;
        self.program_stack.clear();
        self.program_stack.push(("main".to_string(), 0));
        loop {
            let state = self.step()?;
            match state {
                ProgramState::Processing => continue,
                ProgramState::End => return Ok(()),
            };
        }
        Ok(())
    }
    fn step(&mut self) -> Result<ProgramState, Error> {
        let (func_name, index) = self.program_stack.pop().ok_or(Error::Runtime)?;
        let func = self.flow_table.get(&func_name).ok_or(Error::Runtime)?;
        let node = &func.body.get(index);
        match node {
            &Some(ref n) => {
                match &n.instruction {
                    &Instruction::LoadString(ref s) => {
                        self.program_stack.push((func_name, index+1));
                    }
                    _ => {
                        return Err(Error::NotImplementedRuntime(n.span.clone()));
                    }
                };
                Ok(ProgramState::Processing)
            },
            &None => {
                print!("end");
                Ok(ProgramState::End)
            }
        }
    }
}



#[repr(C)]
union Register {
    int: i32,
    float: f32,
    bytes: [u8; 4]
}

#[test]
fn test_register() {
    let mut r = Register { int: 32 };
    unsafe {
        assert_eq!(32, r.bytes[0]);
        assert_eq!(0, r.bytes[1]);
        assert_eq!(0, r.bytes[2]);
        assert_eq!(0, r.bytes[3]);

        r.int = 0xFFFF;
        assert_eq!(0xFF, r.bytes[0]);
        assert_eq!(0xFF, r.bytes[1]);
        assert_eq!(0, r.bytes[2]);
        assert_eq!(0, r.bytes[3]);

        r.int = 5;
        assert_eq!(5, r.bytes[0]);
        assert_eq!(0, r.bytes[1]);
        assert_eq!(0, r.bytes[2]);
        assert_eq!(0, r.bytes[3]);

        r.bytes = [0xFF, 0, 0, 0];
        assert_eq!(0xFF, r.int);

        r.int = -100;
        assert_eq!(156, r.bytes[0]);
        assert_eq!(255, r.bytes[1]);
        assert_eq!(255, r.bytes[2]);
        assert_eq!(255, r.bytes[3]);

        r.bytes = [156, 255, 255, 255];
        assert_eq!(-100, r.int);
    }

}

#[test]
fn simple_calc() {
let code = r#"
int add(int a, int b) {
    return a + b;
}
int main(void) {
    printf("%d", add(1, 2));
}"#;
    let meta = MetaData::new(code.to_string());
    let ast = parse(&meta).unwrap();
    let func_table = Convert::convert_program(&ast).unwrap();
    let mut runtime = Runtime::new(func_table);
    runtime.run().unwrap();
    assert!(false);
}
