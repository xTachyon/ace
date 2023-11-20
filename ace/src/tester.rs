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

struct HwRegs {
    rax: u64,
}

unsafe fn run_test(executable_memory: &[u8; 4096]) {
    let f: unsafe extern "C" fn() = transmute(executable_memory.as_ptr());

    asm!(
        "push rax",
        "push rbx",
        "push rcx",
        "push rdx",
        "push rdi",
        "push rsi",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
    );

    f();

    asm!(
        "push r15",
        "push r14",
        "push r13",
        "push r12",
        "push r11",
        "push r10",
        "push r9",
        "push r8",
        "push rsi",
        "push rdi",
        "push rdx",
        "push rcx",
        "push rbx",
        "push rax",
    );
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
