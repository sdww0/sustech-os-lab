#include <stdio.h>
#include <unistd.h>
#include <wait.h>

int main(int argc, char *argv[])
{
    // Disable buffer in STDOUT
    setvbuf(stdout, NULL, _IONBF, 0);

    printf("Running wait with null user mode program\n");

    int pid = fork();
    if (pid == 0)
    {
        // Child
        printf("Here is children! Doing something dummy...\n");

        // Try to uncomment below!
        // execl("hello_world", "hello_world", "temp", NULL);

        printf("Done!\n");
    }
    else
    {
        // Parent
        printf("Here is parent! Waiting for children with pid %d ...\n", pid);

        int wait_pid = wait(NULL);

        printf("Wait complete! The process pid %d\n", wait_pid);
    }
    return 0;
}