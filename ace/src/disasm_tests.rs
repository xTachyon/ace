use crate::registers::R64;
use crate::{DisasmWriter, Registers};
use std::fmt::Write;
use std::{fmt::Arguments, fs, process::Command};

impl DisasmWriter for String {
    fn write(&mut self, args: Arguments<'_>) {
        write!(self, "{}\n", args).unwrap();
    }
}

fn t(text: &str) -> Registers {
    const ASM_FILE_PATH: &str = "tmp/now.s";
    const BIN_FILE_PATH: &str = "tmp/now.bin";

    let text_new = text.to_string() + "\nhlt\n";

    let _ = fs::remove_dir_all("tmp");
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

    let r = super::run(&bin_correct, &mut output);

    fs::write(ASM_FILE_PATH, &output).unwrap();

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
    assert_eq!(bin_correct, bin_new, "\n{}", output);

    r
}

#[test]
fn sub_mem() {
    let text = "
mov     DWORD [rbp-4], 0
mov     eax, DWORD [rbp-4]
    ";

    t(text);
}

#[test]
fn sub() {
    let text = "
mov ebx, 100
sub ebx, 5
sub ebx, -10
    ";

    t(text);
}

#[test]
fn stack_alloc_ret_1000() {
    let text = "
f:
    push    rbp
    mov     rbp, rsp
    sub     rsp, 920
    mov     DWORD [rbp-4], 0
    jmp     .L2
.L3:
    mov     eax, DWORD [rbp-4]
    mov     edx, eax
    mov     eax, DWORD [rbp-4]
    mov     BYTE [rbp-1040+rax], dl
    add     DWORD [rbp-4], 1
.L2:
    cmp     DWORD [rbp-4], 1023
    jbe     .L3
    movzx   eax, BYTE [rbp-40]
    movsx   eax, al
    leave
    ret
    ";

    t(text);
}

// #[test]
// fn stack_alloc() {
//     let text = "
// f:
//     push    rbp
//     mov     rbp, rsp
//     sub     rsp, 920
//     mov     DWORD [rbp-4], 0
//     jmp     .L2
// .L3:
//     mov     eax, DWORD [rbp-4]
//     mov     edx, eax
//     mov     eax, DWORD[rbp-4]
//     mov     BYTE [rbp-1040+rax], dl
//     add     DWORD [rbp-4], 1
// .L2:
//     cmp     DWORD [rbp-4], 1023
//     jbe     .L3
//     nop
//     leave
//     ret
//     ";

//     t(text);
// }

#[test]
fn pop_64() {
    let text = "
pop rbp
    ";

    t(text);
}

#[test]
fn simple_jump() {
    let text = "
    push    rbp
    mov     rbp, rsp
    mov     eax, edi
    mov     BYTE [rbp-4], al
    cmp     BYTE [rbp-4], 0
    je near .L2
    mov     eax, 5
    jmp near .L3
.L2:
    mov     eax, 10
.L3:
    pop     rbp
    ret
    ";

    t(text);
}

#[test]
fn simple_jump2() {
    let text = "
f:
    push    rbp
    mov     rbp, rsp
    mov     edx, edi
    mov     eax, esi
    mov     BYTE [rbp-4], dl
    mov     BYTE [rbp-8], al
    cmp     BYTE [rbp-4], 0
    je near .L2
    cmp     BYTE [rbp-8], 0
    je near .L2
    mov     eax, 5
    jmp near .L3
.L2:
    mov     eax, 10
.L3:
    pop     rbp
    ret
    ";

    t(text);
}

#[test]
fn mov_m() {
    let text = "
mov bl, 5
mov [rbp-4], bl
    ";

    t(text);
}

#[test]
fn xor_64() {
    let text = "
xor rax, rax
xor rbx, rbx
xor rcx, rcx
xor rdx, rdx
xor rsp, rsp
xor rbp, rbp
xor rdi, rdi
xor rsi, rsi

xor r8, r8
xor r9, r9
xor r10, r10
xor r11, r11
xor r12, r12
xor r13, r13
xor r14, r14
xor r15, r15

xor rax, rbx
xor rbx, rcx
xor rcx, rdx

xor rax, r8
xor r8, r9
    ";

    t(text);
}

#[test]
fn xor_32() {
    let text = "
xor eax, eax
xor ebx, ebx
xor ecx, ecx
xor edx, edx
xor esp, esp
xor ebp, ebp
xor edi, edi
xor esi, esi

xor r8d, r8d
xor r9d, r9d
xor r10d, r10d
xor r11d, r11d
xor r12d, r12d
xor r13d, r13d
xor r14d, r14d
xor r15d, r15d

xor eax, ebx
xor ebx, ecx
xor ecx, edx

xor eax, r8d
xor r8d, r9d
    ";

    t(text);
}

#[test]
fn xor_16() {
    let text = "
xor ax, ax
xor bx, bx
xor cx, cx
xor dx, dx
xor sp, sp
xor bp, bp
xor di, di
xor si, si

xor r8w, r8w
xor r9w, r9w
xor r10w, r10w
xor r11w, r11w
xor r12w, r12w
xor r13w, r13w
xor r14w, r14w
xor r15w, r15w

xor ax, bx
xor bx, cx
xor cx, dx

xor ax, r8w
xor r8w, r9w
    ";

    t(text);
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

    let regs = t(text);
    assert_eq!(regs[R64::RAX].r64(), 1);
    assert_eq!(regs[R64::R15].r64(), 16);
}
