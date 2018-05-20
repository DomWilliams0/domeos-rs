/* entry point */
ENTRY(start)

SECTIONS
{
    /* start sections at 1 MiB */
    . = 1M;

    .boot BLOCK(4K) : ALIGN(4K)
    {
        *(.multiboot)
    }

    .text BLOCK(4K) : ALIGN(4K)
    {
        *(.text)
    }
}
