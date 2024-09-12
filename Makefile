obj-m += mymem.o

all: module test

module:
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) modules

test: mymem_test.c
	gcc -o test mymem_test.c

clean:
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) clean
	rm -f test

run: all
	sudo insmod mymem.ko
	sudo ./test
	sudo rmmod mymem.ko

.PHONY: all clean run