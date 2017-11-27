use std::collections::{HashMap};
use flow_graph::{Func, Convert, Instruction, Node, Type};
use helper::parse;
use meta_data::MetaData;
use symbol_table::SymbolTable;
use error::Error;

type History = Vec<(usize, Option<String>)>;
type VarTable = SymbolTable<String, (usize, History)>;

impl VarTable {
    fn declare_var(&mut self, name: String, addr: usize, line: usize)
            -> Result<(), Error> {
        {
            let scope = self.list.front().ok_or(Error::NoScope)?;
            if scope.contains_key(&name) {
                return Err(Error::AlreadyDeclaredVar);
            }
        }
        let mut history = History::new();
        history.push((line, None));
        self.insert(name, (addr, history));

        Ok(())
    }

    fn get_var_addr(&self, name: &String) -> Option<usize> {
        match self.get(name) {
            None => None,
            Some(v) => {
                let (ref addr, _) = *v;
                Some(*addr)
            }
        }

    }
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
        let address = self.stack_pointer.pop().ok_or(
            Error::Runtime("no stack".to_string()))?;
        let end_addr = address + size;
        self.stack_pointer.push(end_addr);
        while self.memory.len() < end_addr{
            self.memory.push(0);
        }
        Ok(address)
    }
    pub fn load_bytes(&mut self, addr: usize, bytes: Vec<u8>) {
        for (i, byte) in bytes.iter().enumerate() {
            self.memory[i + addr] = *byte;
        }
    }
    pub fn get_bytes(&self, addr: usize, size: usize) -> &[u8] {
        &self.memory[addr..addr+size]
    }

    pub fn load_register(&mut self, addr: usize, register:&Register) {
        unsafe {
            for i in 0..4 {
                self.memory[addr + i] = register.bytes[i];
            }
        }
    }
    pub fn get_register(&self, addr: usize) -> Register {
        let bytes:[u8; 4] = [
            self.memory[addr],
            self.memory[addr+1],
            self.memory[addr+2],
            self.memory[addr+3],
        ];
        Register {
            bytes
        }
    }

}

#[derive(Debug)]
pub enum ProgramState {
    Processing,
    End,
}

pub struct Runtime {
    meta: MetaData,
    flow_table: HashMap<String, Func>,
    register_stack: Vec<Register>,
    memory: Memory,
    program_stack: Vec<(String, usize)>,
    var_table: VarTable,
}


impl Runtime {
    fn new(meta:MetaData, flow_table: HashMap<String, Func>) -> Runtime {
        Runtime {
            meta,
            flow_table,
            register_stack: Vec::new(),
            memory: Memory::new(),
            program_stack: Vec::new(),
            var_table: VarTable::new(),
        }
    }
    fn run(&mut self) -> Result<(), Error> {
        self.flow_table.get("main").ok_or(Error::NoMain)?;
        self.program_stack.clear();
        self.program_stack.push(("main".to_string(), 0));
        self.memory.push_scope();
        self.var_table.push_scope();
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
        let (func_name, index) = self.program_stack.pop()
            .ok_or(Error::Runtime("no program pointer".to_string()))?;
        let func = self.flow_table.get(&func_name)
            .ok_or(Error::Runtime("not defined function".to_string()))?;
        let node = &func.body.get(index);
        match node {
            &Some(ref n) => {
                match &n.instruction {
                    &Instruction::LoadString(ref s) => {
                        let mut bytes = s.clone().into_bytes();
                        bytes.push(0);
                        let addr = self.memory.alloc_stack(bytes.len())?;
                        self.memory.load_bytes(addr, bytes);
                        self.register_stack.push(Register {addr: addr as u32});
                        self.program_stack.push((func_name, index+1));
                    },
                    &Instruction::LoadInt(i) => {
                        self.register_stack.push(Register {int:i});
                        self.program_stack.push((func_name, index+1));
                    },
                    &Instruction::LoadVar(ref var_name) => {
                        let addr = self.var_table.get_var_addr(var_name)
                            .ok_or(Error::Runtime("no var".to_string()))?;
                        let data = self.memory.get_bytes(addr, 4);
                        self.register_stack.push(Register::from(data));
                        self.program_stack.push((func_name, index+1));
                    },
                    &Instruction::FuncCall {ref name, args_size} => {
                        self.memory.push_scope();
                        self.var_table.push_scope();
                        let func  = self.flow_table.get(name).ok_or(
                            Error::Runtime("no function".to_string()))?;
                        let mut params : Vec<&(Type, String)> = func.decl.params.iter().collect();


                        for i in 0..args_size {
                            let addr = self.memory.alloc_stack(4)?;
                            let (_, ref var_name) = *params.pop()
                                .ok_or(Error::Runtime("call error".to_string()))?;
                            self.memory.load_register(
                                addr,
                                &self.register_stack.pop().ok_or(
                                    Error::Runtime("no register".to_string()))?);
                            self.var_table.declare_var(
                                var_name.to_string(),
                                addr,
                                self.meta.line(n.span.lo));

                        }

                        self.program_stack.push((func_name, index+1));
                        self.program_stack.push((name.to_string(), 0));
                    },
                    _ => {
                        return Err(Error::NotImplementedRuntime(
                                format!("{:?}", n),
                                n.span.clone()));
                    },

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
    addr: u32,
    int: i32,
    float: f32,
    bytes: [u8; 4]
}

impl <'a> From<&'a [u8]> for Register {
    fn from(data:&[u8]) -> Register{
        let bytes: [u8; 4] = [data[0], data[1], data[2], data[3]];
        Register { bytes }

    }

}

impl Register {
    fn load_bytes(&mut self, bytes:&[u8]) {
        unsafe {
            for i in 0..4 {
                self.bytes[i] = bytes[i];
            }

        }
    }
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
    let mut runtime = Runtime::new(meta, func_table);
    runtime.run().unwrap();
    assert!(false);
}
