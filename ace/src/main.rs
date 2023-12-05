#[cfg(test)]
mod disasm_tests;
mod registers;
mod tester;

use anyhow::Result;
use std::fmt::Debug;
use std::fmt::Display;
// use registers::R16::*;
// use registers::R32::*;
use registers::Register;
use registers::R16;
use registers::R32;
use registers::R64;
use registers::R64::*;
use registers::R8;

#[repr(align(8))]
#[derive(Clone, Copy, PartialEq)]
struct RegData {
    x: [u8; 8],
}
impl RegData {
    const ZERO: RegData = RegData { x: [0; 8] };

    fn r8(self) -> u8 {
        self.x[0]
    }
    fn r16(self) -> u16 {
        u16::from_ne_bytes([self.x[0], self.x[1]])
    }
    fn r32(self) -> u32 {
        u32::from_ne_bytes([self.x[0], self.x[1], self.x[2], self.x[3]])
    }
    fn r64(self) -> u64 {
        u64::from_ne_bytes(self.x)
    }
    fn set_r8(&mut self, new: u8) {
        self.x[0] = new;
    }
    fn set_r16(&mut self, new: u16) {
        self.x[0..2].copy_from_slice(&new.to_le_bytes());
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

fn push_reg(regs: &mut Registers, stack: &mut [u8], register: R64) {
    let rsp = regs[RSP].r64() as usize;
    let source = regs[register];
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

trait DisasmWriter: Display {
    fn write(&mut self, args: std::fmt::Arguments<'_>);
}

struct Nothing;
impl Display for Nothing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Nothing")
    }
}
impl DisasmWriter for Nothing {
    fn write(&mut self, _args: std::fmt::Arguments<'_>) {}
}

macro_rules! w {
    ($dst:expr, $($arg:tt)*) => {
        $dst.write(std::format_args!($($arg)*))
    };
}

fn xor<R: Register, D: DisasmWriter>(modrm: ModRm, rex: Rex, registers: &mut Registers, d: &mut D) {
    // dbg!(modrm);
    // dbg!(rex);
    let r1 = R::from_index(modrm.rm() + 8 * rex.b() as u8);
    let r2 = R::from_index(modrm.reg() + 8 * rex.r() as u8);

    w!(d, "xor {}, {}", r1, r2);

    let v1 = R::from_reg(registers[r1]);
    let v2 = R::from_reg(registers[r2]);

    let result = v1 ^ v2;

    registers[r1].set_r64(result.into());
}

struct Eflags(u32);
impl Eflags {}

struct Registers([RegData; 16]);

impl<T: Register> std::ops::Index<T> for Registers {
    type Output = RegData;

    fn index(&self, index: T) -> &Self::Output {
        &self.0[index.as_usize()]
    }
}
impl<T: Register> std::ops::IndexMut<T> for Registers {
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        &mut self.0[index.as_usize()]
    }
}

fn run<D: DisasmWriter>(code: &[u8], d: &mut D) -> Registers {
    let mut ip = 0usize;
    let mut registers = Registers([RegData::ZERO; 16]);
    let mut stack = [0u8; 1024 * 1024];

    registers[RBP].set_r64(stack.len() as u64);
    registers[RSP].set_r64(stack.len() as u64);

    let mut rex_prefix: Option<Rex> = None;
    let mut is_16_bit = false;

    loop {
        let opcode = code[ip];
        match opcode {
            0x0f => {
                if code[ip + 1] == 0x84 {
                    // je/jz rel32
                    // 0F 84 cd 	JE rel32 	D 	Valid 	Valid 	Jump near if equal (ZF=1).

                    let rel32 = i32::from_le_bytes([
                        code[ip + 2],
                        code[ip + 3],
                        code[ip + 4],
                        code[ip + 5],
                    ]);

                    w!(d, "je near {}", rel32 + 4);

                    ip += 1 + 1 + 4;
                } else {
                    todo!()
                }
            }
            0x31 => {
                // xor

                let rex = rex_prefix.unwrap_or_default();
                let modrm = ModRm(code[ip + 1]);

                if is_16_bit {
                    xor::<R16, _>(modrm, rex, &mut registers, d);
                } else if rex.w() {
                    xor::<R64, _>(modrm, rex, &mut registers, d);
                } else {
                    xor::<R32, _>(modrm, rex, &mut registers, d);
                }

                ip += 2;
                rex_prefix = None;
            }
            0x55 => {
                // push rbp
                w!(d, "push rbp");
                push_reg(&mut registers, &mut stack, RBP);
                ip += 1;
            }
            0x58..=0x5f => {
                // pop r64
                // 58+ rd 	POP r64 	O 	Valid 	N.E. 	Pop top of stack into r64; increment stack pointer.

                let reg = R64::from_index(opcode - 0x58);

                w!(d, "pop {}", reg);

                ip += 1;
            }
            0x66 => {
                is_16_bit = true;
                ip += 1;
            }
            0x74 => {
                // je/jz rel8
                //  74 cb 	JE rel8 	D 	Valid 	Valid 	Jump short if equal (ZF=1).

                let rel8 = code[1] as i8;

                w!(d, "je {}", rel8);

                ip += 2;
            }
            0x80 => {
                // cmp r/m8, imm8

                let modrm = ModRm(code[ip + 1]);

                let dst = R64::from_index(modrm.rm());
                // let src = R8::from_index(modrm.reg());

                let mod_ = modrm.mod_();
                match mod_ {
                    0b01 => {
                        let disp = code[ip + 2] as i8;
                        let imm = code[ip + 3];

                        w!(d, "cmp byte [{}{:+}], {}", dst, disp, imm);

                        ip += 4;
                    }
                    _ => todo!(),
                }

                rex_prefix = None;
            }
            0x81 => {
                // sub r/m16/32/64, imm16/32
                // 81 /5 iw 	SUB r/m16, imm16 	MI 	Valid 	Valid 	Subtract imm16 from r/m16.

                let rex = rex_prefix.unwrap_or_default();
                let modrm = ModRm(code[ip + 1]);

                // let src = R8::from_index(modrm.reg());

                let mod_ = modrm.mod_();
                match mod_ {
                    0b01 => {
                        todo!()
                        // let disp = code[ip + 2] as i8;
                        // let imm = code[ip + 3];

                        // w!(d, "sub {}, {}", dst, disp);

                        // ip += 4;
                    }
                    0b11 => {
                        if rex.w() {
                            let dst = R64::from_index(modrm.rm());

                            let imm = i32::from_le_bytes([
                                code[ip + 2],
                                code[ip + 3],
                                code[ip + 4],
                                code[ip + 5],
                            ]);

                            let value = registers[dst].r64() as i64 - imm as i64;
                            registers[dst].set_r64(value as u64);

                            w!(d, "sub {}, {}", dst, imm);

                            ip += 1 + 1 + 4;
                        } else {
                            let dst = R32::from_index(modrm.rm());

                            let imm = i32::from_le_bytes([
                                code[ip + 2],
                                code[ip + 3],
                                code[ip + 4],
                                code[ip + 5],
                            ]);

                            let value = registers[dst].r32() as i32 - imm;
                            registers[dst].set_r32(value as u32);

                            w!(d, "sub {}, {}", dst, imm);

                            ip += 1 + 1 + 4;
                        }
                    }
                    _ => todo!("{:?}", modrm),
                }

                rex_prefix = None;
            }
            0x88 => {
                // mov r/m8, r8

                // let rex = rex_prefix.unwrap_or_default();
                let modrm = ModRm(code[ip + 1]);

                let dst = R64::from_index(modrm.rm());
                let src = R8::from_index(modrm.reg());

                let mod_ = modrm.mod_();
                match mod_ {
                    0x01 => {
                        let disp = code[ip + 2] as i8;
                        let addr = registers[dst].r64() as i64 + disp as i64;

                        stack[addr as usize] = registers[src].r8();

                        w!(d, "mov [{}{:+}], {}", dst, disp, src);

                        ip += 3;
                    }
                    _ => todo!(),
                }

                rex_prefix = None;
            }
            0x89 => {
                // mov r,r

                let rex = rex_prefix.unwrap_or_default();
                let modrm = ModRm(code[ip + 1]);

                if is_16_bit {
                    todo!()
                } else if rex.w() {
                    assert_eq!(modrm.mod_(), 0b11);

                    let dst = R64::from_index(modrm.rm());
                    let src = R64::from_index(modrm.reg());

                    w!(d, "mov {}, {}", dst, src);

                    let value = registers[src].r64();
                    registers[dst].set_r64(value);
                } else {
                    assert_eq!(modrm.mod_(), 0b11);

                    let dst = R32::from_index(modrm.rm());
                    let src = R32::from_index(modrm.reg());

                    w!(d, "mov {}, {}", dst, src);

                    let value = registers[src].r32();
                    registers[dst].set_r32(value);
                }

                ip += 2;

                is_16_bit = false;
                rex_prefix = None;
            }
            0x8b => {
                // mov

                let rex = rex_prefix.unwrap_or_default();
                let modrm = ModRm(code[ip + 1]);

                let mod_ = modrm.mod_();
                match mod_ {
                    0b01 => {
                        if rex.w() {
                            todo!()
                        } else {
                            let disp = code[ip + 2] as i8;

                            let src = R64::from_index(modrm.rm());
                            let dst = R32::from_index(modrm.reg());
                            let mem_src = registers[src].r64() as i64 + disp as i64;
                            let mem_src = mem_src as usize;

                            let mut data = [0; 4];
                            data.copy_from_slice(&stack[mem_src..mem_src + 4]);
                            let data = i32::from_le_bytes(data);
                            registers[dst].set_r32(data as u32);

                            w!(d, "mov {}, [{}{:+}]", dst, src, disp);

                            ip += 1 + 1 + 1;
                        }
                    }
                    _ => todo!("{:?}", modrm),
                }

                rex_prefix = None;
            }
            0xb0..=0xb7 => {
                // let rex = rex_prefix.unwrap_or_default();

                let reg = R8::from_index(opcode - 0xb0);
                let data = code[ip + 1];

                registers[reg].set_r8(data);

                w!(d, "mov {}, {}", reg, data);

                ip += 2;
                rex_prefix = None;
            }
            0xb8..=0xbf => {
                // mov r, imm16/32/64

                let rex = rex_prefix.unwrap_or_default();

                if is_16_bit {
                    assert!(code.len() >= ip + 2);

                    let reg = R16::from_index(opcode - 0xb8 + 8 * rex.b() as u8);

                    let data = i16::from_le_bytes([code[ip + 1], code[ip + 2]]);

                    registers[reg].set_r16(data as u16);

                    w!(d, "mov {}, {:#x}", reg, data);

                    ip += 1 + 2;
                } else if rex.w() {
                    assert!(code.len() >= ip + 8);

                    let reg = R64::from_index(opcode - 0xb8 + 8 * rex.b() as u8);

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

                    registers[reg].set_r64(data as u64);

                    w!(d, "mov {}, {:#x}", reg, data);

                    ip += 1 + 8;
                } else {
                    assert!(code.len() >= ip + 5);

                    let reg = R32::from_index(opcode - 0xb8 + 8 * rex.b() as u8);

                    let data = i32::from_le_bytes([
                        code[ip + 1],
                        code[ip + 2],
                        code[ip + 3],
                        code[ip + 4],
                    ]);

                    registers[reg].set_r32(data as u32);

                    w!(d, "mov {}, {:#x}", reg, data);

                    ip += 1 + 4;
                }

                is_16_bit = false;
                rex_prefix = None;
            }
            0xc3 => {
                w!(d, "ret");
                break;
            }
            0xc7 => {
                // mov

                let rex = rex_prefix.unwrap_or_default();
                let modrm = ModRm(code[ip + 1]);

                let mod_ = modrm.mod_();
                match mod_ {
                    0b01 => {
                        if rex.w() {
                            todo!()
                        } else {
                            let disp = code[ip + 2] as i8;
                            let data = i32::from_le_bytes([
                                code[ip + 3],
                                code[ip + 4],
                                code[ip + 5],
                                code[ip + 6],
                            ]);

                            let dst = R64::from_index(modrm.rm());
                            let mem_dst = registers[dst].r64() as i64 + disp as i64;
                            let mem_dst = mem_dst as usize;

                            stack[mem_dst..mem_dst + 4].copy_from_slice(&data.to_le_bytes());

                            w!(d, "mov dword [{}{:+}], {}", dst, disp, data);

                            ip += 1 + 1 + 1 + 4;
                        }
                    }
                    0b11 => {
                        if rex.w() {
                            assert!(code.len() >= ip + 6);
                            let data = [code[ip + 2], code[ip + 3], code[ip + 4], code[ip + 5]];
                            let data = i32::from_le_bytes(data) as i64;

                            let dst = R64::from_index(modrm.reg());
                            registers[dst].set_r64(data as u64);

                            ip += 6;

                            w!(d, "mov {}, {:#x}", dst, data);
                        } else {
                            todo!()
                        }
                    }
                    _ => todo!("{:?}", modrm),
                }

                rex_prefix = None;
            }
            0xe9 => {
                // jmp rel32
                // E9 cd 	JMP rel32 	D 	Valid 	Valid 	Jump near, relative, RIP = RIP + 32-bit displacement sign extended to 64-bits.

                let rel32 =
                    i32::from_le_bytes([code[ip + 1], code[ip + 2], code[ip + 3], code[ip + 4]]);

                w!(d, "jmp near {}", rel32 + 4);

                ip += rel32 as i64 as usize;
            }
            0xf3 => {
                if code[ip + 1..].starts_with(&[0x0f, 0x1e, 0xfa]) {
                    // endbr64
                    w!(d, "endbr64");
                    ip += 4;
                } else {
                    todo!();
                }
            }
            0xf4 => {
                // hlt, we use it for testing as it can never appear in userspace code
                break;
            }
            _ => {
                if opcode & 0b0100 << 4 != 0 {
                    rex_prefix = Some(Rex(opcode));
                    // dbg!(rex_prefix);
                    ip += 1;
                } else {
                    todo!("opcode={:#x}\n+++++++++++++++++++++++++++++++++++\n{}+++++++++++++++++++++++++++++++++++", opcode, d);
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
