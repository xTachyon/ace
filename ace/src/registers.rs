use std::ops::BitXor;

use crate::RegData;

pub trait Register: Copy {
    type BaseType: BitXor<Output = Self::BaseType> + Into<u64>;

    fn from_index(x: u8) -> Self;
    fn as_usize(self) -> usize;
    fn name(self) -> &'static str;
    fn from_reg(x: RegData) -> Self::BaseType;
}

use R16::*;
use R32::*;
use R64::*;

#[derive(Debug, Copy, Clone)]
pub enum R64 {
    RAX,
    RCX,
    RDX,
    RBX,
    RSP,
    RBP,
    RSI,
    RDI,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

// impl R64 {
//     pub fn from_index_consecutive(x: u8) -> R64 {
//         match x {
//             0 => RAX,
//             1 => RBX,
//             2 => RCX,
//             3 => RDX,
//             4 => RDI,
//             5 => RSI,
//             6 => RBP,
//             7 => RSP,
//             //
//             8 => R8,
//             9 => R9,
//             10 => R10,
//             11 => R11,
//             12 => R12,
//             13 => R13,
//             14 => R14,
//             15 => R15,
//             //
//             _ => unreachable!("invalid register number"),
//         }
//     }
// }
impl Register for R64 {
    type BaseType = u64;

    fn from_index(x: u8) -> R64 {
        match x {
            0 => RAX,
            1 => RCX,
            2 => RDX,
            3 => RBX,
            4 => RSP,
            5 => RBP,
            6 => RSI,
            7 => RDI,
            //
            8 => R8,
            9 => R9,
            10 => R10,
            11 => R11,
            12 => R12,
            13 => R13,
            14 => R14,
            15 => R15,
            //
            _ => unreachable!("invalid register number"),
        }
    }

    fn as_usize(self) -> usize {
        self as usize
    }

    fn name(self) -> &'static str {
        match self {
            RAX => "rax",
            RCX => "rcx",
            RDX => "rdx",
            RBX => "rbx",
            RSP => "rsp",
            RBP => "rbp",
            RSI => "rsi",
            RDI => "rdi",
            R8 => "r8",
            R9 => "r9",
            R10 => "r10",
            R11 => "r11",
            R12 => "r12",
            R13 => "r13",
            R14 => "r14",
            R15 => "r15",
        }
    }

    fn from_reg(x: RegData) -> Self::BaseType {
        x.r64()
    }
}

#[derive(Debug, Copy, Clone)]
pub enum R32 {
    EAX,
    ECX,
    EDX,
    EBX,
    ESP,
    EBP,
    ESI,
    EDI,
    R8D,
    R9D,
    R10D,
    R11D,
    R12D,
    R13D,
    R14D,
    R15D,
}

impl Register for R32 {
    type BaseType = u32;

    fn from_index(x: u8) -> R32 {
        match x {
            0 => EAX,
            1 => ECX,
            2 => EDX,
            3 => EBX,
            4 => ESP,
            5 => EBP,
            6 => ESI,
            7 => EDI,
            //
            8 => R8D,
            9 => R9D,
            10 => R10D,
            11 => R11D,
            12 => R12D,
            13 => R13D,
            14 => R14D,
            15 => R15D,
            //
            _ => unreachable!("invalid register number"),
        }
    }

    fn as_usize(self) -> usize {
        self as usize
    }

    fn name(self) -> &'static str {
        match self {
            EAX => "eax",
            EBX => "ebx",
            ECX => "ecx",
            EDX => "edx",
            EDI => "edi",
            ESI => "esi",
            EBP => "ebp",
            ESP => "esp",
            R8D => "r8d",
            R9D => "r9d",
            R10D => "r10d",
            R11D => "r11d",
            R12D => "r12d",
            R13D => "r13d",
            R14D => "r14d",
            R15D => "r15d",
        }
    }

    fn from_reg(x: RegData) -> Self::BaseType {
        x.r32()
    }
}

#[derive(Debug, Copy, Clone)]
pub enum R16 {
    AX,
    CX,
    DX,
    BX,
    SP,
    BP,
    SI,
    DI,
    R8W,
    R9W,
    R10W,
    R11W,
    R12W,
    R13W,
    R14W,
    R15W,
}

impl Register for R16 {
    type BaseType = u16;

    fn from_index(x: u8) -> R16 {
        match x {
            0 => AX,
            1 => CX,
            2 => DX,
            3 => BX,
            4 => SP,
            5 => BP,
            6 => SI,
            7 => DI,
            //
            8 => R8W,
            9 => R9W,
            10 => R10W,
            11 => R11W,
            12 => R12W,
            13 => R13W,
            14 => R14W,
            15 => R15W,
            //
            _ => unreachable!("invalid register number"),
        }
    }

    fn as_usize(self) -> usize {
        self as usize
    }

    fn name(self) -> &'static str {
        match self {
            AX => "ax",
            BX => "bx",
            CX => "cx",
            DX => "dx",
            DI => "di",
            SI => "si",
            BP => "bp",
            SP => "sp",
            R8W => "r8w",
            R9W => "r9w",
            R10W => "r10w",
            R11W => "r11w",
            R12W => "r12w",
            R13W => "r13w",
            R14W => "r14w",
            R15W => "r15w",
        }
    }

    fn from_reg(x: RegData) -> Self::BaseType {
        x.r16()
    }
}
