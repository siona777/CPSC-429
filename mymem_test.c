#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>
#include <string.h>
#include <time.h>


#define DEVICE_PATH "/dev/mymem"  // Path to the device file
#define NUM_TRIALS 10
#define BILLION 1000000000L

void test_open_close(int *fd) {
    *fd = open(DEVICE_PATH, O_RDWR);
    if (*fd < 0) {
        perror("Failed to open the device");
        exit(EXIT_FAILURE);
    } else {
        printf("Device opened successfully: %d\n", *fd);
    }

    if (close(*fd) < 0) {
        perror("Failed to close the device");
        exit(EXIT_FAILURE);
    } else {
        printf("Device closed successfully\n");
    }
}

void test_read(int fd) {
    char buffer[128];
    memset(buffer, 0, sizeof(buffer));

    if (read(fd, buffer, sizeof(buffer)) < 0) {
        perror("Failed to read from the device");
        exit(EXIT_FAILURE);
    } else {
        printf("Read data: %s\n", buffer);
    }
}

void test_write(int fd) {
    const char *data = "Hello from userspace!";
    if (write(fd, data, strlen(data)) < 0) {
        perror("Failed to write to the device");
        exit(EXIT_FAILURE);
    } else {
        printf("Wrote data: %s\n", data);
    }
}

void measure_write_time(int fd, size_t size) {
    char *to_write = malloc(size);
    memset(to_write, 'A', size);
    struct timespec start, end;
    double total_time = 0;

    printf("test write for size : %lu\n", size);
    for(int i = 0; i < NUM_TRIALS; i++) {
        lseek(fd, 0, SEEK_SET);  // Reset file offset for each trial
        clock_gettime(CLOCK_MONOTONIC, &start); // Start time
        if (write(fd, to_write, size) < 0) {
            free(to_write);
            perror("could not write to file");
            exit(EXIT_FAILURE);
        }

        clock_gettime(CLOCK_MONOTONIC, &end);  // End time
        double elapsed_time = (end.tv_sec - start.tv_sec) +
                      (end.tv_nsec - start.tv_nsec) / (double) BILLION;
        total_time += elapsed_time;
        printf("%f\n", elapsed_time);
    }
    
    double avg_time = total_time / NUM_TRIALS;
    printf("Average write time for %zu bytes: %.9f seconds\n", size, avg_time);
    free(to_write);
}

void measure_read_time(int fd, size_t size) {
    char *buffer = malloc(size);
    struct timespec start, end;
    double total_time = 0;

    printf("test read for size : %lu\n", size);
    for (int i = 0; i < NUM_TRIALS; i++) {
        lseek(fd, 0, SEEK_SET);  // Reset file offset for each trial
        clock_gettime(CLOCK_MONOTONIC, &start);  // Start time
        if (read(fd, buffer, size) < 0) {
            perror("Failed to read from the device");
            free(buffer);
            exit(EXIT_FAILURE);
        }
        clock_gettime(CLOCK_MONOTONIC, &end);  // End time
        double elapsed_time = (end.tv_sec - start.tv_sec) +
                      (end.tv_nsec - start.tv_nsec) / (double) BILLION;
        total_time += elapsed_time;
        printf("%f\n", elapsed_time);
    }

    double avg_time = total_time / NUM_TRIALS;
    free(buffer);
}

void measure_read_and_write_time(int fd, size_t size) {
    char *buffer = malloc(size);
    char *to_write = malloc(size);
    memset(to_write, 'A', size);
    struct timespec start, end;
    double total_time = 0;

    for (int i = 0; i < NUM_TRIALS; i++) {
        lseek(fd, 0, SEEK_SET);  // Reset file offset for each trial
        clock_gettime(CLOCK_MONOTONIC, &start);  // Start time
        if (write(fd, to_write, size) < 0) {
            perror("Failed to write to the device");
            free(buffer);
            free(to_write);
            exit(EXIT_FAILURE);
        }

        if (lseek(fd, 0, SEEK_SET) == (off_t)-1) {  // Reset file offset before reading
            perror("Failed to reset file offset before read");
            break;
        }

        if (read(fd, buffer, size) < 0) {
            perror("Failed to read from the device");
            free(buffer);
            free(to_write);
            exit(EXIT_FAILURE);
        }
        clock_gettime(CLOCK_MONOTONIC, &end);  // End time
        double elapsed_time = (end.tv_sec - start.tv_sec) +
                      (end.tv_nsec - start.tv_nsec) / (double) BILLION;
        total_time += elapsed_time;
        printf("%f\n", elapsed_time);
    }

    double avg_time = total_time / NUM_TRIALS;
    free(buffer);
}


int main() {
    int fd;
    size_t sizes[] = {64, 1024, 65536, 524288};  // Sizes: 1 byte, 64 bytes, 1KB, 64KB, 512KB

    // Open the device file
    fd = open(DEVICE_PATH, O_RDWR);
    if (fd < 0) {
        perror("Failed to open the device");
        return errno;
    }

    // Measure write time
    measure_write_time(fd, 1);

    // Measure read time
    measure_read_time(fd, 1);

    for (int i = 0; i < sizeof(sizes) / sizeof(sizes[0]); i++) {
        size_t size = sizes[i];
        printf("\nTesting with %zu bytes:\n", size);

        measure_read_and_write_time(fd, size);
    }

    // Close the device file
    if (close(fd) < 0) {
        perror("Failed to close the device");
        return errno;
    }

    return 0;
}