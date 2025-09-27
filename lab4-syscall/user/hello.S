
.global _start
.section .text
_start:
    li     a7, 64             
    li     a0, 1               
    la     a1, message            
    la     a2, message_end
    sub    a2, a2, a1            
    ecall
    # TODO-1.3: Replace "93" with the new syscall number
    li     a7, 93             
    li     a0, 0               
    ecall
    li     a7, 93              
    li     a0, 0               
    ecall  

.section .rodata                  
message:
    .ascii  "Hello, world\n"
message_end: