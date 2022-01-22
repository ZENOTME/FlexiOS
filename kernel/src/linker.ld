KERNEL_BASE = 0xffff000000000000;
ADDR_BASE = 0x80000;
ENTRY(_start)
SECTIONS {
  
  . = ADDR_BASE; /* Load the kernel at this address. */
  .init : {
    sinit = .;
    *(.text.boot)
  }
  .init_data : {
    *(.stack.boot)
  }
  . = ALIGN(4K);
  einit = ABSOLUTE(.);
  skernel = einit;

  .text KERNEL_BASE+skernel: AT(skernel){
    stext = .-KERNEL_BASE;
    
    *(.text .text.* .gnu.linkonce.t*)
    . = ALIGN(4K);
    etext = .-KERNEL_BASE;
  }

  .rodata : {
    srodata = .-KERNEL_BASE;
    *(.rodata .rodata.* .gnu.linkonce.r*)
    . = ALIGN(4K);
    erodata = .-KERNEL_BASE;
  }

  .data : {
    sdata = .-KERNEL_BASE;
    *(.data .data.* .gnu.linkonce.d*)
    . = ALIGN(4K);
    edata = .-KERNEL_BASE;
  }

  .stack : {
    . = ALIGN(4K);
    *(.bss.stack)
  }

  .bss : {
    . = ALIGN(32);
    sbss = .-KERNEL_BASE;
    *(.bss .bss.*)
    *(COMMON)
    . = ALIGN(4K);
    ebss = .-KERNEL_BASE;
  }

  /* end of the binary */
  ekernel = ALIGN(8)-KERNEL_BASE;

  /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}