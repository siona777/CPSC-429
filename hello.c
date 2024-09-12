#include <linux/init.h>
#include <linux/module.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Siona Tagare");
MODULE_DESCRIPTION("Hello World Kernel Module");

//initialize – print hello world
static int __init hello_init(void) {
    printk(KERN_INFO "Hello, World!\n");
    return 0;
}

//exit – print goodbye world
static void __exit hello_exit(void) {
    printk(KERN_INFO "Goodbye, World!\n");
}

module_init(hello_init);
module_exit(hello_exit);