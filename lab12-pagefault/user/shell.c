#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <stdio.h>

#define NUM 1024

int execute(char *command)
{
    if (strcmp(command, "exit") == 0)
    {
        exit(0);
    }

    pid_t pid = fork();
    if (pid < 0)
    {
        perror("Fork failed");
        return -1;
    }
    else if (pid == 0)
    {
        execl(command, NULL);
        exit(EXIT_FAILURE);
    }
    else
    {
        int status;
        waitpid(pid, &status, 0);
    }
    return 0;
}

int main()
{
    // Disable buffer in STDOUT
    setvbuf(stdout, NULL, _IONBF, 0);
    printf("Running Shell...\n");
    while (1)
    {
        char command[NUM];
        printf("~ # ");
        char *cmd = fgets(command, NUM, stdin);
        command[strlen(cmd) - 1] = '\0';
        printf("\n Running command: %s\n", command);
        execute(command);
    }
}