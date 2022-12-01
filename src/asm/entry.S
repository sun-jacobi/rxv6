        # qemu -kernel loads the kernel at 0x80000000
        # and causes each hart (i.e. CPU) to jump there.
        # kernel.ld causes the following code to
        # be placed at 0x80000000.
.section .text
.global _entry
_entry:
        # set up a stack for C.
        # stack0 is declared in start.c,
        # with a 4096-byte stack per CPU.  
        la sp, stack0 
        li a0, 4096
        csrr a1, mhartid
        addi a1, a1, 1
        mul a0, a0, a1 
        add sp, sp, a0 # sp = stack0 + (hartid * 4096)
        call start # jump to start() in start.c
spin:
        j spin

.section .data
.align 4 
stack0:
        # create stack0
        .space 4096*4
