#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <sys/wait.h>
#include <sys/mman.h>

#define ALLOC_SIZE (100 * 1024 * 1024)
#define FORK_COUNT 100

int main()
{
    // Disable buffer in STDOUT
    setvbuf(stdout, NULL, _IONBF, 0);
    
    struct timespec start, end;

    printf("Starting Anonymous Mmap Lazy Allocation Test...\n");

    // 1. Use mmap to allocate 100MB of anonymous memory
    void *ptr = mmap(NULL, ALLOC_SIZE, PROT_READ | PROT_WRITE,
                     MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);

    if (ptr == MAP_FAILED)
    {
        perror("mmap failed");
        return 1;
    }
    printf("Mmapped 100MB at %p\n", ptr);

    clock_gettime(CLOCK_MONOTONIC, &start);

    // 3. Multiple forks to test lazy allocation behavior
    for (int i = 0; i < FORK_COUNT; i++)
    {
        int pid = fork();
        if (pid < 0)
        {
            perror("fork failed");
            exit(1);
        }
        else if (pid == 0)
        {
            exit(0);
        }
    }

    for (int i = 0; i < FORK_COUNT; i++)
    {
        wait(NULL);
    }

    clock_gettime(CLOCK_MONOTONIC, &end);

    long seconds = end.tv_sec - start.tv_sec;
    long nanoseconds = end.tv_nsec - start.tv_nsec;
    long ms = seconds * 1000 + nanoseconds / 1000000;

    printf("Time elapsed for %d forks with 100MB unmapped memory: %ld ms\n", FORK_COUNT, ms);

    return 0;
}
