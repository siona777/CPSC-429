obj-m += mymem.o

all: module test

module:
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) modules

test: multiple_threads.c
	gcc -o test multiple_threads.c

clean:
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) clean
	rm -f test

run: all
	sudo insmod mymem.ko
	sudo ./test
	sudo rmmod mymem.ko

.PHONY: all clean run