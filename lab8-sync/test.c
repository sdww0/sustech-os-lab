// test.c

#include <threads.h>
#include <stdio.h>

unsigned long counter = 0;
const unsigned long N = 1000000;

void thread_func()
{
    while (counter < N)
    {
        counter++;
    }
}

int main()
{
    thrd_t thread[20];

    for (int i = 0; i < 20; i++)
    {
        thrd_create(&thread[i], (thrd_start_t)thread_func, NULL);
    }

    for (int i = 0; i < 20; i++)
    {
        thrd_join(thread[i], NULL);
    }

    printf("Final counter value: %lu\n", counter);

    return 0;
}