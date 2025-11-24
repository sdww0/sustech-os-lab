#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/mman.h>
#include <sys/stat.h>

int main() {
    const char *filename = "hello";
    const char *content = "Hello World";
    size_t content_size = strlen(content);
    
    // Step 1: Create file and write data
    printf("1. Creating file and writing data...\n");
    int fd = open(filename, O_RDWR | O_CREAT | O_TRUNC, 0644);
    if (fd == -1) {
        perror("Failed to open file");
        exit(1);
    }

    // Write data to file
    if (write(fd, content, content_size) != content_size) {
        perror("Failed to write file");
        close(fd);
        exit(1);
    }
    
    printf("Written: %s\n", content);
    
    // Step 2: Map file to memory using mmap
    printf("\n2. Mapping file to memory using mmap...\n");
    
    // Choose a fixed virtual address (typically available on most systems)
    void *fixed_addr = (void *)0x30000000;
    size_t map_size = content_size;
    
    // Use MAP_FIXED to force mapping at specified address
    void *mapped_addr = mmap(fixed_addr, map_size, 
                            PROT_READ, 
                            MAP_PRIVATE | MAP_FIXED, 
                            fd, 0);
    
    if (mapped_addr == MAP_FAILED) {
        perror("mmap failed");
        close(fd);
        exit(1);
    }
    
    printf("Successfully mapped file to fixed address: %p\n", mapped_addr);
    
    // Step 3: Read data from memory-mapped region
    printf("\n3. Reading data from memory-mapped region...\n");
    
    // Read directly from memory, no file I/O operations needed
    char buffer[64];
    strncpy(buffer, (char*)mapped_addr, content_size);
    buffer[content_size] = '\0';  // Ensure string termination
    
    printf("Content read from memory mapping: %s\n", buffer);
    
    // Verify content correctness
    if (strcmp(buffer, content) == 0) {
        printf("✓ Verification successful: Memory content matches original file\n");
    } else {
        printf("✗ Verification failed: Memory content mismatch\n");
    }
    
    // Step 4: Clean up resources
    printf("\n4. Cleaning up resources...\n");
    
    if (munmap(mapped_addr, map_size) == -1) {
        perror("munmap failed");
    } else {
        printf("Successfully unmapped memory\n");
    }
    
    close(fd);
    
    return 0;
}
