global long_mode_start
extern  _start

section .text
bits 64
long_mode_start:

; run kernel
;push    ebx
call    _start

; print message instead
mov		rax, 0x2f592f412f4b2f4f
mov		qword [0xb8000], rax

; hang on exit
cli
jmp     $

