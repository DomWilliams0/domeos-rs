global long_mode_start

section .text
bits 64
long_mode_start:

; run kernel
;extern  kernel_main
;push    ebx
;call    kernel_main

; print message instead
mov		rax, 0x2f592f412f4b2f4f
mov		qword [0xb8000], rax

; hang on exit
cli
jmp     $

