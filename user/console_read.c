#include <stdio.h>

int main(int argc, char *argv[])
{
    // Disable buffer in STDOUT
    setvbuf(stdout, NULL, _IONBF, 0);

    printf("Ready to receive console input!\n");

    char input[64];
    scanf("%s", input);

    printf("Here is your input: %s\n", input);
}
