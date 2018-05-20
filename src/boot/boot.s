global start
extern long_mode_start

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


section .rodata
gdt64:
dq		0										; first null entry
.cs: equ $ - gdt64
dq		(1<<43) | (1<<44) | (1<<47) | (1<<53)	; code, data not needed
												; flags set: descriptor type, present, exec, 64 bit
.ptr:
dw		$ - gdt64 - 1							; length of gdt
dq		gdt64

section .text
bits	 32

; entry point
start:

; setup stack
mov     esp, stack_top

; paging
call	init_page_tables
call	enable_paging

; load 64 bit gdt
lgdt	[gdt64.ptr]

; jump into 64 bit mode!
jmp		gdt64.cs:long_mode_start

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
