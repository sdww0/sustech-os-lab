#include <stdio.h>
#include <unistd.h>

int some_func()
{
    printf("Hello~");
}

int main()
{
    // Disable buffer in STDOUT
    setvbuf(stdout, NULL, _IONBF, 0);

    long illegal_addr = (long)some_func;

    printf("illegal_addr: %x\n", illegal_addr);

    // We shouldn't write something into the code section.
    *(int *)illegal_addr = 0;

    printf("illegal page fault test failed!\n");
}
