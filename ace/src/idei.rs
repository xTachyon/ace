mov_style! {
    "mov", (r/r, r/m, m/r, r/imm), 16/32/64
}

match opcode {
    0x88 => mov_style!("mov", r/m, 16/32/64, "MOV"),
    0xadd => mov_style!("add", r/m, 16/32/64, "ADD"),
}

"MOV" => 
```
#[inline(always)]
fn MOV<T: Sized + Copy, const DISASM: bool>(src: &T, dst: &mut T) {
    if DISASM {
        ...
    } else {
        *dst = *src;
    }
}
```

------------------------------------------------------------

struct Opt {
    src: Memory | Imm | R,
    dst: Memory | Imm | R,
}

match opcode {
    0x88 => {
        let opt = mov_style!("mov", r/m, 16/32/64, "MOV"),       
    }
}

enum Instr { Add(..), Mov(..) }
fn get_next_instr() -> Instr;

------------------------------------------------------------

struct TwoParams<T> {
    src: *const T,
    dst: *mut T,
}

fn/macro analyze_two_params(buffer: &[u8], mem: &Memory) -> TwoParams {}

match opcode {
    0x88 => {
        let params = analyze_two_params(..);
        MOV(params);
    }
    jump cond => {
        if DISASM {
            ip += bytes instr;
        } else {
            ip += disp;
        }
    }
}

------------------------------------------------------------

enum Prefix {
    // rax-rsp
    P16S,
    P32S,
    P64S,
    // r8 - r15
    P16E,
    P32E,
    P64E,
}

match opcode {
    0x66 => prefix = P16,
    0x67 => prefix = ..
    0x68 => prefix = ..
    _ => match prefix {
        P16 => instructiuni de 16 match [opcode + 1] {
            // doar instructiunile de 16
        },
        P32 => instructiuni de 32 match [opcode + 1] {
            // doar instructiunile de 32
        },
        P64 => instructiuni de 64 match [opcode + 1] {
            // doar instructiunile de 64
        },
    }
}

------------------------------------------------------------

enum I {
    Add16, ....
    Uncached
}

------------------------------------------------------------

add [ebx * 4 + ecx], 5
