use crate::gdb::{Debuggable, Message, GDB};
use anyhow::anyhow;
use anyhow::Result;
use std::{
    fs::{self, File},
    io::Read,
    process::Command,
};

trait ProcSpawnOk {
    fn spawn_ok(&mut self) -> Result<()>;
}
impl ProcSpawnOk for Command {
    fn spawn_ok(&mut self) -> Result<()> {
        let status = self.status()?;
        if status.success() {
            Ok(())
        } else {
            Err(anyhow!("oof process did an oopsie"))
        }
    }
}

#[derive(Default, Copy, Clone)]
#[repr(C)]
struct HwRegs {
    regs: [u64; 16],
}

fn run_one(path: &str, s: &str, tmp: &mut Vec<u8>) -> Result<()> {
    const TO_FIND: &str =
        "-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-";
    const ASM_FILE_PATH: &str = "tmp/now.s";
    const BIN_FILE_PATH: &str = "tmp/now.bin";

    let index = s.find(TO_FIND).expect("no separator found");

    let asm = &s[..index];
    // let conditions = &s[index + TO_FIND.len()..];

    fs::write(ASM_FILE_PATH, &asm)?;

    Command::new("nasm")
        .args([ASM_FILE_PATH, "-felf64", "-O0", "-g"])
        .spawn_ok()?;

    Command::new("objcopy")
        .args(["-O", "binary", "-j", ".text", "tmp/now.o", BIN_FILE_PATH])
        .spawn_ok()?;

    Command::new("ld")
        .args(["tmp/now.o", "-o", "tmp/now"])
        .spawn_ok()?;

    {
        let mut file = File::open(BIN_FILE_PATH)?;
        tmp.clear();
        file.read_to_end(tmp)?;
    }

    assert!(tmp.len() <= 4096);

    // let soft = super::run(&tmp, &mut Nothing);

    let mut gdb = GDB::new("tmp/now");

    while let Some(_) = gdb.recv_async() {}

    let register_names = gdb.register_names();

    let expected_breakpoints = 0;
    for (line, text) in asm.lines().enumerate() {
        if text.is_empty() {
            continue;
        }
        gdb.breakpoint("tmp/now.s", line as u32 + 1);
    }

    gdb.run();

    let mut confirmed_breakpoints = 0;
    while let Some(message) = gdb.recv() {
        match message {
            Message::BreakpointCreated => {
                confirmed_breakpoints += 1;
                if confirmed_breakpoints == expected_breakpoints {
                    break;
                }
            }
            Message::BreakpointHit => {}
            _ => {}
        }
    }

    assert_eq!(confirmed_breakpoints, expected_breakpoints);

    // ignore r15 for now
    // for i in 0..15 {
    //     if i == R64::RSP as usize || i == R64::RBP as usize {
    //         continue;
    //     }
    //     assert_eq!(regs.regs[i], soft.general[i].r64(), "at {}", i);
    // }

    todo!();
    Ok(())
}

struct NewTester {}
impl Debuggable for NewTester {
    fn init(&mut self, gdb: &mut GDB) {
        gdb.breakpoint("main.cpp", 5);
    }
}

pub fn run_impl() -> Result<()> {
    fs::create_dir_all("tmp")?;

    let mut buffer = String::new();
    let mut tmp = Vec::new();
    for i in fs::read_dir("../tests")? {
        let path = i?.path();
        let path = path.to_str().unwrap();
        println!("---------------------- {}", path);

        let mut file = File::open(path)?;

        tmp.clear();
        buffer.clear();
        file.read_to_string(&mut buffer)?;
        run_one(path, &buffer, &mut tmp)?;
    }

    Ok(())
}

pub fn run() {
    run_impl().unwrap();
}
