mod tester;

use anyhow::Result;

enum R64 {
    RAX,
    RBX,
    RCX,
    RDX,
    RDI,
    RSI,
    RBP,
    RSP,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

macro_rules! decl_reg {
    ($name:ident, $($r:ident,)*) => {
        #[derive(Debug, Copy, Clone)]
        enum $name {
            $(
                $r,
            )*
        }
        impl $name {
            fn from_index(x: u8) -> Self {
                $(
                    if x == ($r as u8) {
                        return $r;
                    }
                )*
                unreachable!()
            }
            fn name(self) -> &'static str {
                match self {
                    $(
                        $r => stringify!($r),
                    )*
                }
            }
            fn as_usize(self) -> usize {
                self as usize
            }
        }
    };
}
decl_reg!(R32, EAX, ECX, EDX, EBX, ESP, EBP, ESI, EDI,);
impl R64 {
    fn as_usize(self) -> usize {
        self as usize
    }
    // fn from_index(x: u8) -> Self {
    //     match x {
    //         0 => RAX,
    //         1 => RBX,
    //         2 => RCX,
    //         3 => RDX,
    //         4 => RDI,
    //         5 => RSI,
    //         6 => RBP,
    //         7 => RSP,
    //         8 => R8,
    //         9 => R9,
    //         10 => R10,
    //         11 => R11,
    //         12 => R12,
    //         13 => R13,
    //         14 => R14,
    //         15 => R15,
    //         _ => unimplemented!(),
    //     }
    // }
}

use R32::*;
use R64::*;

#[repr(align(8))]
#[derive(Clone, Copy)]
struct RegData {
    x: [u8; 8],
}
impl RegData {
    const ZERO: RegData = RegData { x: [0; 8] };

    fn r32(self) -> u32 {
        u32::from_ne_bytes([self.x[0], self.x[1], self.x[2], self.x[3]])
    }
    fn r64(self) -> u64 {
        u64::from_ne_bytes(self.x)
    }
    fn set_r32(&mut self, new: u32) {
        self.x = (new as u64).to_ne_bytes();
    }
    fn set_r64(&mut self, new: u64) {
        self.x = new.to_ne_bytes();
    }
}

fn push_reg(regs: &mut [RegData; 16], stack: &mut [u8], register: R64) {
    let rsp = regs[RSP.as_usize()].r64() as usize;
    let source = regs[register.as_usize()];
    stack[rsp - 8..rsp].copy_from_slice(&source.x);
}

fn run(code: &[u8]) {
    let mut ip = 0;
    let mut registers = [RegData::ZERO; 16];
    let mut stack = [0u8; 1024 * 1024];

    registers[RBP.as_usize()].set_r64(stack.len() as u64);
    registers[RSP.as_usize()].set_r64(stack.len() as u64);

    loop {
        let opcode = code[ip];
        match opcode {
            0x55 => {
                // push rbp
                println!("push rbp");
                push_reg(&mut registers, &mut stack, RBP);
                ip += 1;
            }
            0x89 => {
                let info = code[ip + 1];
                let dst = R32::from_index(info & 0b111);
                let src = R32::from_index((info >> 3) & 0b111);
                let address = (info >> 6) & 0b11;
                assert_eq!(address, 0b11);

                println!("mov {}, {}", dst.name(), src.name());

                registers[dst.as_usize()].set_r32(registers[src.as_usize()].r32());

                ip += 2;
            }
            0xc3 => {
                println!("ret");
                todo!()
            }
            0xf3 => {
                if code[ip + 1..].starts_with(&[0x0f, 0x1e, 0xfa]) {
                    // endbr64
                    println!("endbr64");
                    ip += 4;
                } else {
                    unimplemented!();
                }
            }
            _ => unimplemented!(),
        }
    }
}

fn main() -> Result<()> {
    // let data = fs::read("binfile")?;
    // run(&data);

    tester::run()?;

    Ok(())
}
