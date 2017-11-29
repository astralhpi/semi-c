use error::Error;
use flow_graph::Type;

#[repr(C)]
pub union Register {
    pub addr: u32,
    pub int: i32,
    pub float: f32,
    pub ch: char,
    pub bytes: [u8; 4]
}

impl <'a> From<&'a [u8]> for Register {
    fn from(data:&[u8]) -> Register{
        let bytes: [u8; 4] = [data[0], data[1], data[2], data[3]];
        Register { bytes }

    }

}

impl Register {
    pub fn load_bytes(&mut self, bytes:&[u8]) {
        unsafe {
            for i in 0..4 {
                self.bytes[i] = bytes[i];
            }

        }
    }
    pub fn stringify(&self, t: &Type) -> String {
        unsafe {
            match t {
                &Type::Int => format!("{}", self.int),
                &Type::Char => format!("{}", self.bytes[0] as char),
                &Type::Float => format!("{:.4}", self.float),
                &Type::Pointer(_) => format!("{:#x}", self.addr),
                _ => format!("N/A")
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
        r.int = 100;
        assert_eq!(r.addr, 100);
    }

}
