
.global _start
.section .text
_start:
    li     a7, 2
    li     a0, 0x1234
    li     a1, 0x5678
    ecall

    li     a7, 60
    mv     a0, a1
    ecall

.section .rodata                  
message:
    .ascii  "Hello, world\n"
message_end: