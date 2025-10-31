// test2.c

#include <threads.h>
#include <stdio.h>
#include <stdatomic.h>

atomic_ulong counter = 0;
const unsigned long N = 1000000;

void thread_func()
{
    unsigned long current;
    while (1)
    {
        current = atomic_load(&counter);
        if (current >= N)
            break;
        atomic_compare_exchange_weak(&counter, &current, current + 1);
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
