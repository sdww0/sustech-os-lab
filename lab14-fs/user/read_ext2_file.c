#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <fcntl.h>
#include <string.h>

int main() {
    int fd1, fd2;
    pid_t pid;
    // Child process
    char buffer[100];

    // Open "hello" and read data
    fd1 = open("hello.txt", O_WRONLY, 0644);
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

    close(fd1);


    return 0;
}