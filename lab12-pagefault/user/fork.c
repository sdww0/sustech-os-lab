#include <stdio.h>
#include <unistd.h>

int main()
{
    printf("Hello Fork!, my pid: %d\n", getpid());
    fflush(stdout);
    
    if (fork() == 0)
    {
        // Child process
        printf("Hello from Child Process!, my pid: %d\n", getpid());
        printf("[Child] My Parent's pid: %d\n", getppid());
    }
    else
    {
        // Parent process
        printf("Hello from Parent Process!, my pid: %d\n", getpid());
        printf("[Parent] My Parent's pid: %d\n", getppid());
    }
}
