section .text
    global _start
    
_start:
    xor eax, eax
    xor ebx, ebx
    xor ecx, ecx
    xor edx, edx
    xor r8d, r8d

    xor eax, ebx
    xor ebx, ecx
    xor ecx, edx

    xor eax, r8d
    xor r8d, r9d
    hlt

-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
