.global _start
.section .text
_start:
    ebreak

    li      a7, 93              
    li      a0, 0       
    ecall  

.section .rodata                  
message:
    .ascii  "Hello, world\n"
message_end: