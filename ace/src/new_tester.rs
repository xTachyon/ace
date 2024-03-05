use crate::gdb::RegisterNames;
use crate::gdb::{Debuggable, Message, GDB};
use crate::registers::{Register, R64};
use crate::{Emulator, Nothing};
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

fn process_register_names(names: RegisterNames) -> [R64; 16] {
    let mut regs = [R64::R15; 16];

    for (index, reg) in names.inner.into_iter().enumerate().take(16) {
        regs[index] = R64::from_name(&reg);
    }

    regs
}

fn map_registers(mapping: &[R64; 16], regs: Vec<(u8, u64)>) -> [u64; 16] {
    let mut result = [0; 16];
    for (number, value) in regs {
        let number = number as usize;
        if number >= mapping.len() {
            continue;
        }
        let index = mapping[number];
        result[index as usize] = value;
    }
    result
}

fn run_one(s: &str, tmp: &mut Vec<u8>) -> Result<()> {
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
    let register_table = process_register_names(register_names);

    // for (line, text) in asm.lines().enumerate() {
    //     if text.is_empty() {
    //         continue;
    //     }
    //     gdb.breakpoint("tmp/now.s", line as u32 + 1);
    // }
    gdb.breakpoint_fn("_start");

    gdb.run();

    let Some(hlt_line) = asm.lines().position(|x| x.contains("hlt")) else {
        panic!("hlt instruction not found");
    };
    let hlt_line = hlt_line as u32 + 1;

    let mut d = Nothing;
    let mut emulator = Emulator::new(tmp, &mut d);

    let mut first = true;
    'end: loop {
        while let Some(message) = gdb.recv() {
            if let Message::BreakpointHit { line } | Message::EndSteppingRange { line } = message {
                if line == hlt_line {
                    break 'end;
                }
                break;
            }
        }

        let registers = gdb.registers();
        let registers = map_registers(&register_table, registers);

        gdb.step();

        if first {
            first = false;
            continue;
        }

        emulator.run();

        for i in 0..16 {
            if i == 4 || i == 5 {
                continue;
            }
            let hw_value = registers[i];
            let soft_value = emulator.regs.general[i].r64();

            assert_eq!(
                hw_value,
                soft_value,
                "at {}({})",
                R64::from_index(i as u8),
                i
            );
        }
    }

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
        run_one(&buffer, &mut tmp)?;
    }

    Ok(())
}

pub fn run() {
    run_impl().unwrap();
}
