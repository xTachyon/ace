use anyhow::Result;
use libc::{
    mmap, MAP_ANONYMOUS, MAP_FAILED, MAP_FIXED, MAP_PRIVATE, PROT_EXEC, PROT_NONE, PROT_READ,
    PROT_WRITE,
};
use std::arch::asm;
use std::io::Write;
use std::mem::transmute;
use std::ptr::null_mut;
use std::{
    fs::{self, File},
    io::Read,
    process::Command,
};

unsafe fn map_x_memory() -> &'static mut [u8; 4096] {
    let page_size = 4096;
    let region_size = 4 * page_size;

    let base_address = mmap(
        null_mut(),
        region_size,
        PROT_NONE,
        MAP_PRIVATE | MAP_ANONYMOUS,
        -1,
        0,
    );

    if base_address == MAP_FAILED {
        panic!("mmap failed");
    }

    let executable_address = mmap(
        base_address.add(page_size),
        region_size - 2 * page_size,
        PROT_READ | PROT_WRITE | PROT_EXEC,
        MAP_FIXED | MAP_PRIVATE | MAP_ANONYMOUS,
        -1,
        0,
    );

    if executable_address == MAP_FAILED {
        panic!("mmap failed");
    }

    let memory = executable_address as *mut [u8; 4096];
    &mut *memory
}

#[derive(Default, Copy, Clone)]
#[repr(C)]
struct HwRegs {
    regs: [u64; 14]
}
macro_rules! r {
    ($i:ident, $e:expr) => {
        fn $i(&self) -> u64 {
            self.regs[$e]
        }
    };
}
impl HwRegs {
    r!(rax, 0);
    r!(rbx, 1);
    r!(rcx, 2);
    r!(rdx, 3);
}

// mov [r15], rax
// mov [r15+8], rbx
// mov [r15+16], rcx
// mov [r15+24], rdx
// mov [r15+32], rdi
// mov [r15+40], rsi
// mov [r15+48], r8
// mov [r15+56], r9
// mov [r15+64], r10
// mov [r15+72], r11
// mov [r15+80], r12
// mov [r15+88], r13
// mov [r15+96], r14

unsafe extern "sysv64" fn run_test(executable_memory: &[u8; 4096]) -> HwRegs {
    let f: unsafe extern "C" fn() = transmute(executable_memory.as_ptr());
    let mut regs = HwRegs::default();

    asm!(
        "
        push rsp
        push rbp
        push rbx
        push r12
        push r13
        push r14
        push r15
        
        push {0}
        call {1}
        pop r15

        mov [r15], rax
        mov [r15+8], rbx
        mov [r15+16], rcx
        mov [r15+24], rdx
        mov [r15+32], rdi
        mov [r15+40], rsi
        mov [r15+48], r8
        mov [r15+56], r9
        mov [r15+64], r10
        mov [r15+72], r11
        mov [r15+80], r12
        mov [r15+88], r13
        mov [r15+96], r14

        pop r15
        pop r14
        pop r13
        pop r12
        pop rbx
        pop rbp
        pop rsp
        ",
        in(reg) &mut regs,
        in(reg) f,
    );

    regs
}

fn run_one(s: &str, tmp: &mut Vec<u8>, executable_memory: &mut [u8; 4096]) -> Result<()> {
    const TO_FIND: &str =
        "-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-";
    const ASM_FILE_PATH: &str = "tmp/now.s";
    const BIN_FILE_PATH: &str = "tmp/now.bin";

    let index = s.find(TO_FIND).expect("no separator found");

    let asm = &s[..index];
    let conditions = &s[index + TO_FIND.len()..];

    write!(tmp, "BITS 64\n{}\nret", asm)?;
    fs::write(ASM_FILE_PATH, &tmp)?;

    Command::new("nasm")
        .args([ASM_FILE_PATH, "-felf64"])
        .status()?;

    Command::new("objcopy")
        .args(["-O", "binary", "-j", ".text", "tmp/now.o", BIN_FILE_PATH])
        .status()?;

    {
        let mut file = File::open(BIN_FILE_PATH)?;
        tmp.clear();
        file.read_to_end(tmp)?;
    }

    assert!(tmp.len() <= 4096);

    executable_memory[0..tmp.len()].copy_from_slice(tmp);
    executable_memory[tmp.len()..].fill(0xcc);

    let regs = unsafe { run_test(&executable_memory) };

    let soft = super::run(executable_memory);

    assert_ne!(regs.rax(), soft)

    Ok(())
}

pub fn run() -> Result<()> {
    fs::create_dir_all("tmp")?;

    let executable_memory = unsafe { map_x_memory() };

    let mut buffer = String::new();
    let mut tmp = Vec::new();
    for i in fs::read_dir("../tests")? {
        let mut file = File::open(i?.path())?;

        buffer.clear();
        file.read_to_string(&mut buffer)?;
        run_one(&buffer, &mut tmp, executable_memory)?;
    }

    Ok(())
}
