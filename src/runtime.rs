use std::collections::{HashMap};
use flow_graph::{Func, Convert, Instruction, Node, Type};
use helper::parse;
use meta_data::MetaData;
use symbol_table::SymbolTable;
use error::Error;
use register::Register;
use memory::Memory;
use printf::Printf;

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
    pub stdout: String,
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
            stdout: String::new()
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
        if self.program_stack.len() == 0 {
            assert_eq!(self.register_stack.len(), 1);
            return Ok(ProgramState::End);
        }

        let (func_name, index) = self.program_stack.pop().unwrap();
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
                        self.register_stack.push(Register {int: i});
                        self.program_stack.push((func_name, index+1));
                    },
                    &Instruction::LoadFloat(f) => {
                        self.register_stack.push(Register {float: f});
                        self.program_stack.push((func_name, index+1));
                    },
                    &Instruction::LoadChar(c) => {
                        self.register_stack.push(Register {ch: c});
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
                            let (_, ref var_name) = *params.pop().ok_or(
                                Error::Runtime("call error".to_string()))?;
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
                    &Instruction::Return => {
                        self.memory.drop_scope();
                        self.var_table.drop_scope();
                    },
                    &Instruction::ReturnVoid => {
                        self.register_stack.push(Register {int: 0});
                        self.memory.drop_scope();
                        self.var_table.drop_scope();
                    },
                    &Instruction::Printf { ref args_size } => {
                        let len = self.register_stack.len();
                        let regs: Vec<_> = self.register_stack.drain(
                            len-args_size..).collect();
                        match Printf::printf(&regs, &self.memory) {
                            Some(s) => {
                                self.stdout.push_str(&s);
                                print!("{}", &s);
                            },
                            None => {
                                return Err(Error::Runtime(
                                        "print error".to_string()));
                            }

                        }
                        self.register_stack.push(Register {int: 0});
                        self.program_stack.push((func_name, index+1));
                    },
                    &Instruction::Pop => {
                        self.register_stack.pop();
                        self.program_stack.push((func_name, index+1));

                    },
                    &Instruction::Subi => {
                        let right = self.register_stack.pop().ok_or(
                            Error::Runtime("no register".to_string()))?;
                        let left = self.register_stack.pop().ok_or(
                            Error::Runtime("no register".to_string()))?;
                        unsafe {
                            let val = left.int - right.int;
                            self.register_stack.push(Register {int: val});
                        }
                        self.program_stack.push((func_name, index+1));
                    },
                    &Instruction::Addi => {
                        let right = self.register_stack.pop().ok_or(
                            Error::Runtime("no register".to_string()))?;
                        let left = self.register_stack.pop().ok_or(
                            Error::Runtime("no register".to_string()))?;
                        unsafe {
                            let val = left.int + right.int;
                            self.register_stack.push(Register {int: val});
                        }
                        self.program_stack.push((func_name, index+1));
                    },
                    &Instruction::Muli => {
                        let right = self.register_stack.pop().ok_or(
                            Error::Runtime("no register".to_string()))?;
                        let left = self.register_stack.pop().ok_or(
                            Error::Runtime("no register".to_string()))?;
                        unsafe {
                            let val = left.int * right.int;
                            self.register_stack.push(Register {int: val});
                        }
                        self.program_stack.push((func_name, index+1));
                    },
                    &Instruction::Divi => {
                        let right = self.register_stack.pop().ok_or(
                            Error::Runtime("no register".to_string()))?;
                        let left = self.register_stack.pop().ok_or(
                            Error::Runtime("no register".to_string()))?;
                        unsafe {
                            let val = left.int / right.int;
                            self.register_stack.push(Register {int: val});
                        }
                        self.program_stack.push((func_name, index+1));
                    },
                    &Instruction::Minusi => {
                        let mut operand = self.register_stack.pop().ok_or(
                            Error::Runtime("no register".to_string()))?;
                        unsafe {
                            operand.int = - operand.int;
                        }
                        self.register_stack.push(operand);
                        self.program_stack.push((func_name, index+1));
                    },
                    &Instruction::Subf => {
                        let right = self.register_stack.pop().ok_or(
                            Error::Runtime("no register".to_string()))?;
                        let left = self.register_stack.pop().ok_or(
                            Error::Runtime("no register".to_string()))?;
                        unsafe {
                            let val = left.float - right.float;
                            self.register_stack.push(Register {float: val});
                        }
                        self.program_stack.push((func_name, index+1));
                    },
                    &Instruction::Addf => {
                        let right = self.register_stack.pop().ok_or(
                            Error::Runtime("no register".to_string()))?;
                        let left = self.register_stack.pop().ok_or(
                            Error::Runtime("no register".to_string()))?;
                        unsafe {
                            let val = left.float + right.float;
                            self.register_stack.push(Register {float: val});
                        }
                        self.program_stack.push((func_name, index+1));
                    },
                    &Instruction::Mulf => {
                        let right = self.register_stack.pop().ok_or(
                            Error::Runtime("no register".to_string()))?;
                        let left = self.register_stack.pop().ok_or(
                            Error::Runtime("no register".to_string()))?;
                        unsafe {
                            let val = left.float * right.float;
                            self.register_stack.push(Register {float: val});
                        }
                        self.program_stack.push((func_name, index+1));
                    },
                    &Instruction::Divf => {
                        let right = self.register_stack.pop().ok_or(
                            Error::Runtime("no register".to_string()))?;
                        let left = self.register_stack.pop().ok_or(
                            Error::Runtime("no register".to_string()))?;
                        unsafe {
                            let val = left.float / right.float;
                            self.register_stack.push(Register {float: val});
                        }
                        self.program_stack.push((func_name, index+1));
                    },
                    &Instruction::Minusf => {
                        let mut operand = self.register_stack.pop().ok_or(
                            Error::Runtime("no register".to_string()))?;
                        unsafe {
                            operand.float = - operand.float;
                        }
                        self.register_stack.push(operand);
                        self.program_stack.push((func_name, index+1));
                    },
                    &Instruction::CharToInt => {
                        let mut operand = self.register_stack.pop().ok_or(
                            Error::Runtime("no register".to_string()))?;
                        unsafe {
                            operand.int = operand.bytes[0] as i32;
                        }
                        self.register_stack.push(operand);
                        self.program_stack.push((func_name, index+1));
                    },
                    &Instruction::IntToFloat => {
                        let mut operand = self.register_stack.pop().ok_or(
                            Error::Runtime("no register".to_string()))?;
                        unsafe {
                            operand.float = operand.int as f32;
                        }
                        self.register_stack.push(operand);
                        self.program_stack.push((func_name, index+1));
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


#[cfg(test)]
mod tests {
    use super::*;
    fn run_test(code: &str) -> String {
        let meta = MetaData::new(code.to_string());
        let ast = parse(&meta).unwrap();
        let func_table = Convert::convert_program(&ast).unwrap();
        print!("{:?}\n", func_table);
        let mut runtime = Runtime::new(meta, func_table);
        runtime.run().unwrap();
        runtime.stdout.clone()
    }

    #[test]
    fn hello_world() {
        let code = r#"
        int main(void) {
            printf("Hello World!\n");
        }"#;
        assert_eq!(run_test(code), "Hello World!\n");
    }
    #[test]
    fn simple_format() {
        let code = r#"
            int main(void) {
                printf("%d\n", 12313);
                printf("%f\n", 45.0);
                printf("%c\n", 'a');
            }
            "#;
        assert_eq!(run_test(code), "12313\n45.0000\na\n");
    }

    // BONUS
    #[test]
    fn bonus_format() {
        let code = r#"
            int main(void) {
                printf("Hello %s\n", "World!");
            }
            "#;
        assert_eq!(run_test(code), "Hello World!\n");

    }
    #[test]
    fn simple_int_calc() {
        let code = r#"
            int sub(int a, int b) {
                return a - b;
            }
            int add(int c, int d) {
                return c + d;
            }
            int mul(int e, int f) {
                return e * f;
            }
            int div(int g, int f) {
                return g / f;
            }
            int main(void) {
                printf("%d\n", sub(102, 32));
                printf("%d\n", sub(33, 132));
                printf("%d\n", add(33, 44));
                printf("%d\n", add(33, -44));
                printf("%d\n", mul(25, 4));
                printf("%d\n", mul(-5, 20));
                printf("%d\n", div(25, 5));
                printf("%d\n", div(123, -123));
            }"#;
        assert_eq!(run_test(code), indoc!("
            70
            -99
            77
            -11
            100
            -100
            5
            -1
            "));
    }

    #[test]
    fn simple_float_calc() {
    let code = r#"
    float sub(float a, float b) {
        return a - b;
    }
    float add(float c, float d) {
        return c + d;
    }
    float mul(float e, float f) {
        return e * f;
    }
    float div(float g, float f) {
        return g / f;
    }
    int main(void) {
        printf("%f\n", sub(102, 32));
        printf("%f\n", sub(33, 132));
        printf("%f\n", add(33, 44));
        printf("%f\n", add(33, -44));
        printf("%f\n", mul(25, 4));
        printf("%f\n", mul(-5, 20));
        printf("%f\n", div(1, 5));
        printf("%f\n", div(50, -100));
    }"#;
        assert_eq!(run_test(code), indoc!("
        70.0000
        -99.0000
        77.0000
        -11.0000
        100.0000
        -100.0000
        0.2000
        -0.5000
        "));
}

}


