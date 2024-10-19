#include <stdio.h>
#include <unistd.h>
int main(int argc, char *argv[])
{
    setvbuf(stdout, NULL, _IONBF, 0);
    printf("This is testing\n");

    int sum = 0;
    for (int a = 0; a < 100; a++)
    {
        sum += a;
    }

    printf("A1111, sum: %d\n", sum);
    int val1 = fork();
    printf("B2222, val1: %d\n", val1);
    int val2 = fork();
    printf("C3333, val2: %d\n", val2);
    return 0;
}