#include <stdio.h>
#include <unistd.h>
int main(int argc, char *argv[])
{
    // Disable buffer in STDOUT
    setvbuf(stdout, NULL, _IONBF, 0);

    printf("My PID: %d\n", getpid());

    printf("This is before exec\n");

    execl("hello_world", "hello_world", "temp", NULL);

    printf("This is after exec\n");
    return 0;
}