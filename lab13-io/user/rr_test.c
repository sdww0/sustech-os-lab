#include <time.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/time.h>
#include <unistd.h>
#include <wait.h>

static int TOTAL = 10;
static int EXIT_SECONDS = 10;

void delay()
{
    // Do something...
    long a = 0;
    for (long i = 0; i < 1000000000000; i++)
    {
        a += 1;
    }
}

int main()
{
    // Disable buffer in STDOUT
    setvbuf(stdout, NULL, _IONBF, 0);
    struct timespec start_time = {0, 0};
    clock_gettime(CLOCK_MONOTONIC, &start_time);

    for (int i = 0; i < TOTAL; i++)
    {
        if (fork() == 0)
        {
            int count = 0;

            while (1)
            {
                delay();
                count += 1;
                struct timespec current = {0, 0};
                clock_gettime(CLOCK_MONOTONIC, &current);
                if (current.tv_sec - start_time.tv_sec >= EXIT_SECONDS)
                {
                    exit(count);
                }
            }
        }
    }

    printf("main: fork ok, now need to wait pids.\n");
    for (int i = 0; i < TOTAL; i++)
    {
        int status = -1;
        int pid = wait(&status);
        printf("main: pid %d, count %d\n", pid, status);
    }
    printf("main: wait pids over\n");
}