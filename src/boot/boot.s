global start

; allocate petit stack of 16KiB
section .bss
align   4
stack_bottom:
        resb 16384
stack_top:


section .text
bits	 32

; entry point
start:

; setup stack
mov     esp, stack_top

; run kernel
;extern  kernel_main
;push    ebx
;call    kernel_main
mov		dword [0xb8000], 0x2f4b2f4a

; hang on exit
cli
jmp     $

.end:
