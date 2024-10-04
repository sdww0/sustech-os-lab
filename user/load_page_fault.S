
.global _start
.section .text
_start:
    li      a1, 0xdeadbeef
    lw      a2, 0(a1)

    li      a7, 60              
    li      a0, 0               
    ecall  

.section .rodata                  
message:
    .ascii  "Hello, world\n"
message_end: