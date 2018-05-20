global start

section .bss
align   4096

; page tables
p4_table:
	resb 4096
p3_table:
	resb 4096
p2_table:
	resb 4096

; allocate petit stack of 16KiB
stack_bottom:
	resb 64
stack_top:


section .text
bits	 32

; entry point
start:

; setup stack
mov     esp, stack_top

; paging
call	init_page_tables
call	enable_paging

; run kernel
;extern  kernel_main
;push    ebx
;call    kernel_main
mov		dword [0xb8000], 0x2f4b2f4a

; hang on exit
cli
jmp     $

init_page_tables:

; first P4 -> p3
mov		eax, p3_table
or		eax, 0b11		; present + w
mov		[p4_table], eax

; first P3 -> p2
mov		eax, p2_table
or		eax, 0b11		; present + w
mov		[p3_table], eax

; map all p2 entries to 2MiB entries
mov		ecx, 0

_map_loop:
mov		eax, 0x200000	; 2MiB
mul		ecx
or		eax, 0b10000011 ; present + w + huge
mov		[p2_table + ecx * 8], eax

inc		ecx
cmp		ecx, 512
jne		_map_loop

ret

enable_paging:

; put p4 in cr3
mov		eax, p4_table
mov		cr3, eax

; pae (bit 5)
mov		eax, cr4
or		eax, 1 << 5
mov		cr4, eax

; long bit in EFER
mov		ecx, 0xC0000080
rdmsr
or		eax, 1 << 8
wrmsr

; paging bit
mov		eax, cr0
or		eax, 1 << 31
mov		cr0, eax

ret


.end:
