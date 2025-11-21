#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <sys/reboot.h>
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
        int child_pid = wait(NULL);
        if (child_pid > 0)
        {
            printf("[INIT] Catch child process, pid: %d\n", child_pid);
        }
        if (child_pid == pid)
        {
            printf("[INIT] Shell process exited, exiting system...\n");
            reboot(RB_POWER_OFF);
        }
    }

    return 0;
}