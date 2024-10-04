
.global _start
.section .text
_start:
    li     a7, 1             
    li     a0, 1               
    la     a1, message            
    la     a2, message_end
    sub    a2, a2, a1            
    ecall
    li     a7, 60              
    li     a0, 0               
    ecall  

.section .rodata                  
message:
    .ascii  "Hello, world\n"
message_end: