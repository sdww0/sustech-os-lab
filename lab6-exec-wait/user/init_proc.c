#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <stdio.h>

int main()
{
    // Disable buffer in STDOUT
    setvbuf(stdout, NULL, _IONBF, 0);
    printf("[INIT] Starting Shell...\n");

    int pid = fork();
    if (pid == 0)
    {
        execl("shell", "shell", NULL);
        exit(1);
    }

    // Do waiting to recycle all of the child process
    while (1)
    {
        int pid = wait(NULL);
        if (pid > 0)
        {
            printf("[INIT] Catch child process, pid: %d\n", pid);
        }
    }

    return 0;
}