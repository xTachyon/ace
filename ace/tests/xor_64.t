section .text
    global _start
    
_start:
    xor rax, rax
    xor rbx, rbx
    xor rcx, rcx
    xor rdx, rdx
    xor r8, r8

    xor rax, rbx
    xor rbx, rcx
    xor rcx, rdx

    xor rax, r8
    xor r8, r9
    hlt

-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
