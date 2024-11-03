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
        printf("Exit shell");
        exit(0);
    }

    pid_t id = fork();

    if (id == -1)
        return -1;

    else if (id == 0)
    {
        execl(command, NULL);
        exit(0);
    }
    else
    {
        int status = 0;
        pid_t rid = waitpid(id, &status, 0);
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