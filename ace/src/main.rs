mod registers;
mod tester;

use anyhow::Result;
use std::fmt::Debug;
// use registers::R16::*;
// use registers::R32::*;
use registers::Register;
use registers::R16;
use registers::R32;
use registers::R64;
use registers::R64::*;

#[repr(align(8))]
#[derive(Clone, Copy)]
struct RegData {
    x: [u8; 8],
}
impl RegData {
    const ZERO: RegData = RegData { x: [0; 8] };

    fn r16(self) -> u16 {
        u16::from_ne_bytes([self.x[0], self.x[1]])
    }
    fn r32(self) -> u32 {
        u32::from_ne_bytes([self.x[0], self.x[1], self.x[2], self.x[3]])
    }
    fn r64(self) -> u64 {
        u64::from_ne_bytes(self.x)
    }
    fn set_r16(&mut self, new: u16) {
        self.x = (new as u64).to_ne_bytes();
    }
    fn set_r32(&mut self, new: u32) {
        self.x = (new as u64).to_ne_bytes();
    }
    fn set_r64(&mut self, new: u64) {
        self.x = new.to_ne_bytes();
    }
}
impl Debug for RegData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegData").field("x", &self.r64()).finish()
    }
}

fn push_reg(regs: &mut [RegData; 16], stack: &mut [u8], register: R64) {
    let rsp = regs[RSP.as_usize()].r64() as usize;
    let source = regs[register.as_usize()];
    stack[rsp - 8..rsp].copy_from_slice(&source.x);
}

#[derive(Clone, Copy, Default)]
struct Rex(u8);
#[allow(dead_code)]
impl Rex {
    fn b(self) -> bool {
        self.0 & 0b1 != 0
    }
    fn x(self) -> bool {
        self.0 & 0b10 != 0
    }
    fn r(self) -> bool {
        self.0 & 0b100 != 0
    }
    fn w(self) -> bool {
        self.0 & 0b1000 != 0
    }
}
impl Debug for Rex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rex")
            .field("b", &self.b())
            .field("x", &self.x())
            .field("r", &self.r())
            .field("w", &self.w())
            .finish()
    }
}

#[derive(Clone, Copy)]
struct ModRm(u8);
impl ModRm {
    fn mod_(self) -> u8 {
        (self.0 >> 6) & 0b11
    }
    fn reg(self) -> u8 {
        (self.0 >> 3) & 0b111
    }
    fn rm(self) -> u8 {
        self.0 & 0b111
    }
}
impl Debug for ModRm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModRm")
            .field("mod", &self.mod_())
            .field("reg", &self.reg())
            .field("rm", &self.rm())
            .finish()
    }
}

fn xor<R: Register>(modrm: ModRm, rex: Rex, registers: &mut [RegData; 16]) {
    let r1 = R::from_index(modrm.rm() + 8 * rex.b() as u8);
    let r2 = R::from_index(modrm.reg() + 8 * rex.r() as u8);

    println!("xor {}, {}", r1.name(), r2.name());

    let v1 = R::from_reg(registers[r1.as_usize()]);
    let v2 = R::from_reg(registers[r2.as_usize()]);

    let result = v1 ^ v2;

    registers[r1.as_usize()].set_r64(result.into());
}

fn run(code: &[u8]) -> [RegData; 16] {
    let mut ip = 0;
    let mut registers = [RegData::ZERO; 16];
    let mut stack = [0u8; 1024 * 1024];

    registers[RBP.as_usize()].set_r64(stack.len() as u64);
    registers[RSP.as_usize()].set_r64(stack.len() as u64);

    let mut rex_prefix: Option<Rex> = None;

    loop {
        let opcode = code[ip];
        match opcode {
            0x31 => {
                // xor

                let modrm = ModRm(code[ip + 1]);

                if let Some(rex) = rex_prefix {
                    if rex.w() {
                        xor::<R64>(modrm, rex, &mut registers);
                    } else {
                        xor::<R16>(modrm, rex, &mut registers);
                    }
                } else {
                    xor::<R32>(modrm, Rex::default(), &mut registers);
                }
                
                ip += 2;
                rex_prefix = None;
            }
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
            0xb8..=0xbf => {
                // mov

                let rex = rex_prefix.unwrap_or_default();

                let reg = R64::from_index(opcode - 0xb8 + 8 * rex.b() as u8);

                if rex.w() {
                    let data = i64::from_le_bytes([
                        code[ip + 1],
                        code[ip + 2],
                        code[ip + 3],
                        code[ip + 4],
                        code[ip + 5],
                        code[ip + 6],
                        code[ip + 7],
                        code[ip + 8],
                    ]);

                    registers[reg.as_usize()].set_r64(data as u64);

                    println!("mov {}, {:#x}", reg.name(), data);

                    ip += 1 + 8;
                } else {
                    assert!(code.len() >= ip + 5);

                    let data = i32::from_le_bytes([
                        code[ip + 1],
                        code[ip + 2],
                        code[ip + 3],
                        code[ip + 4],
                    ]);

                    registers[reg.as_usize()].set_r32(data as u32);

                    println!("mov {}, {:#x}", reg.name(), data);

                    ip += 1 + 4;
                }

                rex_prefix = None;
            }
            0xc3 => {
                println!("ret");
                break;
            }
            0xc7 => {
                // mov
                let modrm = ModRm(code[1]);
                assert_eq!(modrm.mod_(), 0b11);

                let rex = rex_prefix.unwrap_or_default();
                if rex.w() {
                    assert!(code.len() >= ip + 6);
                    let data = [code[ip + 2], code[ip + 3], code[ip + 4], code[ip + 5]];
                    let data = i32::from_le_bytes(data) as i64;

                    let dst = R64::from_index(modrm.reg());
                    registers[dst.as_usize()].set_r64(data as u64);

                    ip += 6;

                    println!("mov {}, {:#x}", dst.name(), data);
                } else {
                    todo!()
                }

                rex_prefix = None;
            }
            0xf3 => {
                if code[ip + 1..].starts_with(&[0x0f, 0x1e, 0xfa]) {
                    // endbr64
                    println!("endbr64");
                    ip += 4;
                } else {
                    todo!();
                }
            }
            _ => {
                if opcode & 0b0100 << 4 != 0 {
                    rex_prefix = Some(Rex(opcode));
                    ip += 1;
                } else {
                    todo!("opcode={:#x}", opcode);
                }
            }
        }
    }

    registers
}

fn main() -> Result<()> {
    // let data = fs::read("binfile")?;
    // run(&data);

    tester::run()?;

    Ok(())
}
