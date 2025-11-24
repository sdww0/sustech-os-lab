#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/time.h>
#include <sys/wait.h>
#include <time.h>

#define NUM_FORKS 100

long long get_current_time_us() {
    struct timespec ts = {0, 0};
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (long long)ts.tv_sec * 1000000 + ts.tv_nsec / 1000;
}

int main() {
    // Disable buffer in STDOUT
    setvbuf(stdout, NULL, _IONBF, 0);

    long long start_time, end_time;
    int i;
    pid_t pid;
    
    printf("Starting fork performance test (%d forks)...\n", NUM_FORKS);
    
    start_time = get_current_time_us();


    for (i = 0; i < NUM_FORKS; i++) {
        pid = fork();
        
        if (pid < 0) {
            perror("fork failed");
            exit(1);
        } else if (pid == 0) {
            exit(0);
        } else {
            wait(NULL);
        }
    }
    
    end_time = get_current_time_us();
    
    long long total_time = end_time - start_time;
    double avg_time = (double)total_time / NUM_FORKS;
    
    printf("\nResult:\n");
    printf("Total fork count: %d\n", NUM_FORKS);
    printf("Total time: %lld microseconds\n", total_time);
    printf("Average time per fork: %.2f microseconds\n", avg_time);
    printf("Average forks per second: %.2f\n", 1000000.0 / avg_time);
    
    return 0;
}
