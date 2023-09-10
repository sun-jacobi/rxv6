OUTPUT_ARCH( "riscv" )
ENTRY( _entry )

SECTIONS
{
  /*
   * ensure that entry.S / _entry is at 0x80000000,
   * where qemu's -kernel jumps.
   */
  . = 0x80000000;

  .text : {
    *(.text.boot) *(.text .text.*)
    . = ALIGN(4096);
    _trampoline = .;
    *(trampsec)
    . = ALIGN(4096);
    ASSERT(. - _trampoline == 0x1000, "error: trampoline larger than one page");
    PROVIDE(etext = .);
  }

  .rodata : {
    . = ALIGN(16);
    *(.srodata .srodata.*) /* do not need to distinguish this from .rodata */
    . = ALIGN(16);
    *(.rodata .rodata.*)
  }

  .data : {
    . = ALIGN(16);
    *(.sdata .sdata.*) /* do not need to distinguish this from .data */
    . = ALIGN(16);
    *(.data .data.*)
    . = ALIGN(4096);
    PROVIDE(heap_start = .); /* heap */ 
    . += 1024*1024*64;
    PROVIDE(heap_end = .); 
    
    PROVIDE(stack_end = .); /* stack for cpus*/ 
    . += 1024*1024*2; 
    . = ALIGN(4096);
    PROVIDE(stack_start = .);
  }

  .bss : {
    . = ALIGN(16);
    *(.sbss .sbss.*) /* do not need to distinguish this from .bss */
    . = ALIGN(16);
    *(.bss .bss.*)
  }
  
  . = ALIGN(4096);
  PROVIDE(end = .);
}