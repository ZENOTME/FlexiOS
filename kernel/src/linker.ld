KERNEL_BASE = 0xffff000000000000;
ADDR_BASE = 0x40080000;
ENTRY(_start)
SECTIONS {
  
  . = ADDR_BASE+KERNEL_BASE; /* Load the kernel at this address. */
  .init : AT(ADDR_BASE){
    sinit = .;
    *(.text.boot)
  }
  . = ALIGN(4K);
  einit = ABSOLUTE(.);
  skernel = einit;

  .text : {
    stext = .;
    
    *(.text .text.* .gnu.linkonce.t*)
    . = ALIGN(4K);
    etext = .;
  }

  .rodata : {
    srodata = .;
    *(.rodata .rodata.* .gnu.linkonce.r*)
    . = ALIGN(4K);
    erodata = .;
  }

  .data : {
    sdata = .;
    *(.data .data.* .gnu.linkonce.d*)
    . = ALIGN(4K);
    edata = .;
  }

  .stack : {
    . = ALIGN(4K);
    *(.bss.stack)
  }

  .bss : {
    . = ALIGN(32);
    sbss = .;
    *(.bss .bss.*)
    *(COMMON)
    . = ALIGN(4K);
    ebss = .;
  }

  /* end of the binary */
  . = ALIGN(4K);
  ekernel = .-KERNEL_BASE;

  /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}
