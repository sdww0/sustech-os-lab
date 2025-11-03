#include <stdio.h>

#define BUFFER_SIZE 64

int main() {
    // Disable buffer in STDOUT
    setvbuf(stdout, NULL, _IONBF, 0);

    int pipefd[2];
    char buffer[BUFFER_SIZE];
    int pid;

    // Create a pipe
    if (pipe(pipefd) < 0) {
        printf("Failed to create pipe\n");
        exit(1);
    }

    // Fork a child process
    pid = fork();
    if (pid < 0) {
        printf("Fork failed\n");
        exit(1);
    }

    if (pid == 0) {
        // Child process: Reader
        close(pipefd[1]); // Close write end
        printf("Child: Waiting to read from pipe...\n");

        int n = read(pipefd[0], buffer, sizeof(buffer));
        if (n > 0) {
            buffer[n] = '\0';
            printf("Child: Read '%s' from pipe\n", buffer);
        } else {
            printf("Child: Failed to read from pipe\n");
        }

        close(pipefd[0]);
        exit(0);
    } else {
        // Parent process: Writer
        close(pipefd[0]); // Close read end
        printf("Parent: Writing to pipe...\n");

        const char *message = "Hello from parent!";
        int n = write(pipefd[1], message, strlen(message));
        if (n < 0) {
            printf("Parent: Failed to write to pipe\n");
        }

        printf("Parent: Write complete, waiting for child to finish...\n");
        wait(0);

        close(pipefd[1]);
        exit(0);
    }
}