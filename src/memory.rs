use register::Register;
use error::Error;

pub struct Memory {
    pub memory: Vec<u8>,
    pub stack_pointer: Vec<usize>
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
    pub fn get_byte(&self, addr: usize) -> u8 {
        self.memory[addr]
    }

    pub fn load_register(&mut self, addr: usize, register:&Register, size:usize) {
        unsafe {
            for i in 0..size {
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
    pub fn is_allocated(&self, addr: usize) -> bool{
        addr < self.memory.len()
    }
}
