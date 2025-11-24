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
    
    printf("Starting fork+exec performance test (%d forks)...\n", NUM_FORKS);
    
    start_time = get_current_time_us();


    for (i = 0; i < NUM_FORKS; i++) {
        pid = fork();
        
        if (pid < 0) {
            perror("fork failed");
            exit(1);
        } else if (pid == 0) {
            execl("hello_world", "hello_world", "temp", NULL);
            perror("exec failed");
            exit(0);
        } else {
            wait(NULL);
        }
    }
    
    end_time = get_current_time_us();
    
    long long total_time = end_time - start_time;

    printf("\nResult:\n");
    printf("Total fork+exec count: %d\n", NUM_FORKS);
    printf("Total time: %lld microseconds\n", total_time);

    return 0;
}
