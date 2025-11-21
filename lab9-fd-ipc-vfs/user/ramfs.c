#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <fcntl.h>
#include <string.h>

int main() {
    int fd1, fd2;
    pid_t pid;

    // Open "hello" and write "Hello" in it
    fd1 = open("hello", O_CREAT | O_WRONLY, 0644);
    if (fd1 < 0) {
        perror("Failed to open hello");
        return 1;
    }
    if (write(fd1, "Hello", 5) != 5) {
        perror("Failed to write to hello");
        close(fd1);
        return 1;
    }
    close(fd1);

    // Open "world" and write "World" in it
    fd2 = open("world", O_CREAT | O_WRONLY, 0644);
    if (fd2 < 0) {
        perror("Failed to open world");
        return 1;
    }
    if (write(fd2, "World", 5) != 5) {
        perror("Failed to write to world");
        close(fd2);
        return 1;
    }
    close(fd2);

    // Fork a child process
    pid = fork();
    if (pid < 0) {
        perror("Fork failed");
        return 1;
    }

    if (pid == 0) {
        // Child process
        char buffer[100];

        // Open "hello" and print its content
        fd1 = open("hello", O_RDONLY);
        if (fd1 < 0) {
            perror("Failed to open hello in child");
            exit(1);
        }
        int length = read(fd1, buffer, 100);
        if (length < 0) {
            perror("Failed to read from hello in child");
            close(fd1);
            exit(1);
        }
        buffer[length] = '\0';
        printf("Content of hello: %s\n", buffer);
        close(fd1);

        // Open "world" and print its content
        fd2 = open("world", O_RDONLY);
        if (fd2 < 0) {
            perror("Failed to open world in child");
            exit(1);
        }
        length = read(fd2, buffer, 100);
        if (length < 0) {
            perror("Failed to read from world in child");
            close(fd2);
            exit(1);
        }
        buffer[length] = '\0';
        printf("Content of world: %s\n", buffer);
        close(fd2);

        exit(0);
    } else {
        // Parent process
        wait(NULL); // Wait for the child process to finish
    }


    // Write Large data (8192B) to hello
    // Open "hello" and write "Hello" in it
    fd1 = open("hello", O_CREAT | O_WRONLY, 0644);
    if (fd1 < 0) {
        perror("Failed to open hello");
        return 1;
    }

    __uint8_t buffer[8192] = {0};

    if (write(fd1, buffer, 8192) != 8192) {
        perror("Failed to write to hello");
        close(fd1);
        return 1;
    }

    if (read(fd1, buffer, 8192) != 8192) {
        perror("Failed to write to hello");
        close(fd1);
        return 1;
    }


    close(fd1);


    return 0;
}