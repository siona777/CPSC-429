#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>
#include <string.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <pthread.h>

#define DEVICE_PATH "/dev/mymem"
#define INIT_VAL 1ULL
#define W 50
#define N 100000
pthread_mutex_t mutex = PTHREAD_MUTEX_INITIALIZER;

// writes init_val to the memory
void initialize(int fd) {
    unsigned long long int initial_value = INIT_VAL;
    if (write(fd, &initial_value, sizeof(initial_value)) < 0) {
        perror("Failed to write to the device");
        exit(EXIT_FAILURE);
    }

    lseek(fd, 0, SEEK_SET);
}

// code for each worker thread
void *worker_process(void *arg) {
    unsigned long long int *worker_value = malloc(sizeof(unsigned long long int));

    int file_fd = open(DEVICE_PATH, O_RDWR);
    if (file_fd < 0) {
        perror("Failed to open the device");
        pthread_exit(NULL);
    }

    // execute n times
    for (int i = 0; i < N; i++) {
        //lock so other threads cannot access memory while read/write occurs
        pthread_mutex_lock(&mutex);

        //read the current value
        lseek(file_fd, 0, SEEK_SET);
        if (read(file_fd, worker_value, sizeof(*worker_value)) < 0) {
            perror("Failed to read from the device");
            pthread_mutex_unlock(&mutex);
            pthread_exit(NULL);
        }

        lseek(file_fd, 0, SEEK_SET);
        unsigned long long int new_value = *worker_value + 1;

        //write updated value
        if (write(file_fd, &new_value, sizeof(*worker_value)) < 0) {
            perror("Failed to write to the device");
            pthread_mutex_unlock(&mutex);
            pthread_exit(NULL);
        }

        lseek(file_fd, 0, SEEK_SET);

        //unlock so other threads can now access memory
        pthread_mutex_unlock(&mutex);
    }

    free(worker_value);
    close(file_fd);
    pthread_exit(NULL);
}

int main() {
    int fd;
    fd = open(DEVICE_PATH, O_RDWR);
    initialize(fd);
    pthread_mutex_init(&mutex, NULL);

    if (close(fd) < 0) {
        perror("Failed to close the device");
        exit(EXIT_FAILURE);
    }
    
    //create worker threads who will execute worker_process
    pthread_t threads[W];

    for (int i = 0; i < W; i++) {
        if (pthread_create(&threads[i], NULL, worker_process, NULL) != 0) {
            perror("Failed to create thread");
            return EXIT_FAILURE;
        }
    }
    
    // wait for all threads to finish
    for (int i = 0; i < W; i++) {
        pthread_join(threads[i], NULL);
    }

    //read final value
    fd = open(DEVICE_PATH, O_RDWR);
    if (fd < 0) {
        perror("Failed to open the device to read final value");
        return 1;
    }

    unsigned long long int *final_buffer = malloc(sizeof(unsigned long long int));

    if (final_buffer == NULL) {
        perror("Failed to allocate memory");
        close(fd);
        return 1;
    }

    lseek(fd, 0, SEEK_SET);
    if (read(fd, final_buffer, sizeof(unsigned long long int)) < 0) {
        perror("Failed to read from the device");
        exit(EXIT_FAILURE);
    }

    printf("The final buffer value is %llu\n", *final_buffer);
    printf("The final value should be %llu\n", INIT_VAL + (N * W));
    printf("The difference is %llu\n", INIT_VAL + (N * W) - *final_buffer);

    pthread_mutex_destroy(&mutex);
    free(final_buffer);
    
    if (close(fd) < 0) {
        perror("Failed to close the device");
        exit(EXIT_FAILURE);
    }
}