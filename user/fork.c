#include <stdio.h>
#include <unistd.h>
#include <wait.h>

int main(int argc, char *argv[])
{
    // Disable buffer in STDOUT
    setvbuf(stdout, NULL, _IONBF, 0);

    printf("A1111, my pid: %d.\n", getpid());
    int val1 = fork();
    printf("B2222, my pid: %d. val1: %d\n", getpid(), val1);
    int val2 = fork();
    printf("C3333, my pid: %d. val2: %d\n", getpid(), val2);

    if (val2 != 0)
    {
        wait(NULL);
        if (val1 != 0)
        {
            wait(NULL);
        }
    }

    return 0;
}