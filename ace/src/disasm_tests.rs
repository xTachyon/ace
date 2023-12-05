use crate::DisasmWriter;
use std::fmt::Write;
use std::{fmt::Arguments, fs, process::Command};

impl DisasmWriter for String {
    fn write(&mut self, args: Arguments<'_>) {
        write!(self, "{}\n", args).unwrap();
    }
}

fn t(text: &str) {
    const ASM_FILE_PATH: &str = "tmp/now.s";
    const BIN_FILE_PATH: &str = "tmp/now.bin";

    let text_new = text.to_string() + "\nhlt\n";

    fs::remove_dir_all("tmp").unwrap();
    fs::create_dir("tmp").unwrap();
    fs::write("tmp/now.s", text_new).unwrap();

    Command::new("nasm")
        .args([ASM_FILE_PATH, "-felf64", "-O0"])
        .status()
        .unwrap();

    Command::new("objcopy")
        .args(["-O", "binary", "-j", ".text", "tmp/now.o", BIN_FILE_PATH])
        .status()
        .unwrap();

    let bin_correct = fs::read(BIN_FILE_PATH).unwrap();
    let mut output = String::new();

    super::run(&bin_correct, &mut output);

    fs::write(ASM_FILE_PATH, output).unwrap();

    Command::new("nasm")
        .args([ASM_FILE_PATH, "-felf64", "-O0"])
        .status()
        .unwrap();

    Command::new("objcopy")
        .args(["-O", "binary", "-j", ".text", "tmp/now.o", BIN_FILE_PATH])
        .status()
        .unwrap();

    let bin_new = fs::read(BIN_FILE_PATH).unwrap();

    let bin_correct = &bin_correct[..bin_correct.len() - 1];
    assert_eq!(bin_correct, bin_new);
}

#[test]
fn mov_64() {
    let text = "
mov rax, 1
mov rbx, 2
mov rcx, 3
mov rdx, 4
mov rsp, 5
mov rbp, 6
mov rsi, 7
mov rdi, 8
mov r8, 9
mov r9, 10
mov r10, 11
mov r11, 12
mov r12, 13
mov r13, 14
mov r14, 15
mov r15, 16
    ";

    t(text);
}

#[test]
fn mov_32() {
    let text = "
mov eax, 1
mov ebx, 2
mov ecx, 3
mov edx, 4
mov esp, 5
mov ebp, 6
mov esi, 7
mov edi, 8
mov r8d, 9
mov r9d, 10
mov r10d, 11
mov r11d, 12
mov r12d, 13
mov r13d, 14
mov r14d, 15
mov r15d, 16
    ";

    t(text);
}

#[test]
fn mov_16() {
    let text = "
mov ax, 1
mov bx, 2
mov cx, 3
mov dx, 4
mov sp, 5
mov bp, 6
mov si, 7
mov di, 8
mov r8w, 9
mov r9w, 10
mov r10w, 11
mov r11w, 12
mov r12w, 13
mov r13w, 14
mov r14w, 15
mov r15w, 16
    ";

    t(text);
}
