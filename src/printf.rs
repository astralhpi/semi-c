use memory::Memory;
use register::Register;

pub struct Printf {}

impl Printf {
    pub fn printf(args: &[Register], memory:&Memory) -> Option<String>{
        let mut args_stack : Vec<_> = args.iter().collect();
        args_stack.reverse();
        let format = args_stack.pop();
        match format {
            Some(ref r) => unsafe {
                let addr = r.addr;
                match Printf::load_str(addr, memory) {
                    None => None,
                    Some(fmt) => Printf::format(&fmt, &mut args_stack, memory)
                }
            },
            None => None
        }
    }
    fn format(fmt: &str, args_stack: &mut Vec<&Register>, memory:&Memory)
            -> Option<String> {
        let mut buffer = String::new();
        let mut is_format = false;
        for c in fmt.chars() {
            if (is_format) {
                let r = args_stack.pop();
                match r {
                    None => return None,
                    Some(ref r) => unsafe {
                        match c {
                            'i' => buffer.push_str(&format!("{}", r.int)),
                            'd' => buffer.push_str(&format!("{}", r.int)),
                            'f' => buffer.push_str(&format!("{:.4}", r.float)),
                            'c' => buffer.push(r.ch),
                            's' => match Printf::load_str(r.addr, memory) {
                                None => return None,
                                Some(ref s) => buffer.push_str(s),
                            },
                            _ => return None,

                        };
                    }
                }
                is_format = false;
            } else if (c == '%'){
                is_format = true;
            } else {
                buffer.push(c);
            }
        }
        Some(buffer)

    }
    fn load_str(addr: u32, memory:&Memory) -> Option<String> {
        let mut data: Vec<u8> = vec![];
        let mut addr = addr;
        loop {
            if !memory.is_allocated(addr as usize) {
                return None;
            }
            let byte = memory.get_byte(addr as usize);
            if byte == 0 {
                break;
            }
            data.push(byte);
            addr += 1;
        }
        String::from_utf8(data).ok()

    }
}


