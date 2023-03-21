set disassemble-next-line on
set confirm off
add-symbol-file target/riscv64gc-unknown-none-elf/debug/kernel
target remote localhost:1234