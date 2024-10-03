all: run

run:
	sudo insmod rust_mymem_final.ko
	sudo insmod rust_test.ko

	sudo dmesg | tail

	sudo rmmod rust_test.ko
	sudo rmmod rust_mymem_final.ko