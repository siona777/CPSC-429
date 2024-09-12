obj-m += hello.o

all: module

module:
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) modules


run: all
	sudo insmod hello.ko
	sudo dmesg | tail
	sudo rmmod hello.ko

clean:
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) clean