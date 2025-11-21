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

    if (fork())
    {
        printf("[Reparent] Parent process exit, pid: %d\n", getpid());
        sched_yield();
        return 0;
    }

    printf("[Reparent] Child process, pid: %d, ppid: %d\n", getpid(), getppid());

    int yield_time = 2;
    printf("[Reparent] Child process yielding, pid: %d\n", getpid());
    for (int i = 0; i < yield_time; i++)
    {
        sched_yield();
    }

    printf("[Reparent] Child process reparenting to init, ppid: %d\n", getppid());
    return 0;
}