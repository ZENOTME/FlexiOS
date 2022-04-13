OUTPUT_ARCH(aarch64)
ENTRY(_start)

BASE_ADDRESS = 0x80400000;

SECTIONS
{
    . = BASE_ADDRESS;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }
    .rodata : ALIGN(0x1000){
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
    .data : ALIGN(0x1000){
        *(.data .data.*)
        *(.sdata .sdata.*)
    }
    .bss : ALIGN(0x1000){
        start_bss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        end_bss = .;
    }
    /DISCARD/ : {
        *(.eh_frame)
        *(.debug*)
    }
}
