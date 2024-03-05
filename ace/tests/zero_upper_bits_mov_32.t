section .text
    global _start
    
_start:
    mov rax, 0xFFFFFFFFFFFFFFFF
    mov ebx, 5
    mov eax, ebx
    hlt

-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

rax == 5
rbx == 5