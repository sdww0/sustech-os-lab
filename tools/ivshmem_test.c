#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/mman.h>
#include <assert.h>

#define SHM_SIZE 4096

int main(int argc, char **argv)
{
    char *p;
    int fd;
    int i;

    fd = open("/dev/shm/shm_qemu_riscv", O_RDWR);
    assert(fd != -1);

    p = mmap(0, SHM_SIZE, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
    assert(p != NULL);

    printf("%s", p);

    munmap(p, SHM_SIZE);
    close(fd);

    return 0;
}