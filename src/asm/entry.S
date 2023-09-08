        # qemu -kernel loads the kernel at 0x80000000
        # and causes each hart (i.e. CPU) to jump there.
        # kernel.ld causes the following code to
        # be placed at 0x80000000.
.section .text.boot
.global _entry
_entry:
        # set up a stack for C.
        # stack0 is declared in start.c,
        # with a 4096-byte stack per CPU.  
        la sp, stack_end
        li a0, 1024*1024*2 # 2MiB 
        csrr a1, mhartid
        addi a1, a1, 1 
        mul a0, a0, a1 # sp = stack_end + (hartid + 1)*2MiB 
        add sp, sp, a0 # the top of each stack
        call start # jump to start() in start.c
spin:
        j spin
