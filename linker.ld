ENTRY(_start)

SECTIONS {
    . = 1M;

    .rodata : {
        /* Ensure the multiboot header is at the very beginning */
        KEEP(*(.multiboot_header))
        *(.rodata .rodata.*)
    }

    .text : { *(.text .text.*) }
    .data : { *(.data .data.*) }
    .bss  : { *(.bss .bss.*) }
}