#include <linux/module.h>
#include <linux/fs.h>
#include <linux/cdev.h>
#include <linux/device.h>
#include <linux/slab.h>
#include <linux/uaccess.h>
#include <linux/fs.h>

#define DEVICE_NAME "mymem"
#define CLASS_NAME "mymem_class"
#define BUFFER_SIZE 524288

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Siona Tagare");
MODULE_DESCRIPTION("Memory Driver Module");

static char *device_buffer;
static int major_number;
size_t buffer_size = 0;
static struct class *custom_class = NULL;
static struct device *custom_device = NULL;

// Function prototypes
static int dev_open(struct inode *, struct file *);
static int dev_release(struct inode *, struct file *);
static ssize_t dev_read(struct file *, char __user *, size_t, loff_t *);
static ssize_t dev_write(struct file *, const char __user *, size_t, loff_t *);
static loff_t dev_llseek(struct file *, loff_t, int);

static struct file_operations fops = {
    .owner = THIS_MODULE,
	.read = dev_read,
	.write = dev_write,
	.open = dev_open,
    .llseek = dev_llseek,
	.release = dev_release
};

//kernel module init
static int __init mymem_init(void) {
    // get a major number to identify the driver
    major_number = register_chrdev(0, DEVICE_NAME, &fops);
    if (major_number < 0) {
        printk(KERN_ALERT "Registering char device failed with %d\n", major_number);
        return major_number;
    }

    // create a class for the driver
    custom_class = class_create(CLASS_NAME);
    if (IS_ERR(custom_class)) {
        unregister_chrdev(major_number, DEVICE_NAME);
        printk(KERN_ALERT "mymem: Failed to register device class\n");
        return PTR_ERR(custom_class);
    }

    // create and register a device for the driver
    custom_device = device_create(custom_class, NULL, MKDEV(major_number, 0), NULL, DEVICE_NAME, 0666);
    if (IS_ERR(custom_device)) {
        class_destroy(custom_class);
        unregister_chrdev(major_number, DEVICE_NAME);
        printk(KERN_ALERT "mymem: Failed to create the device\n");
        return PTR_ERR(custom_device);
    }

    //make space for the 512KB buffer
    device_buffer = kmalloc(BUFFER_SIZE, GFP_KERNEL);

    //if the buffer was not initialized, destroy everything and exit
    if (!device_buffer) {
        device_destroy(custom_class, MKDEV(major_number, 0));
        class_destroy(custom_class);
        unregister_chrdev(major_number, DEVICE_NAME);
        printk(KERN_ALERT "mymem: Failed to allocate memory\n");
        return -ENOMEM;
    }

    return 0;
}

//
static void __exit mymem_exit(void) {
    kfree(device_buffer);
    device_destroy(custom_class, MKDEV(major_number, 0));
    class_unregister(custom_class);
    class_destroy(custom_class);
    unregister_chrdev(major_number, DEVICE_NAME);
}

// read: reads the specified number of bytes starting from the current offset
static ssize_t dev_read(struct file *fptr, char *user_buffer, size_t length, loff_t *offset) {
    //checks that user params are within buffer size
    if ((*offset > BUFFER_SIZE) || ((*offset + length) > BUFFER_SIZE)) {
        return -EINVAL;
    }
    
    //reads bytes from kernel space to user space
    size_t bytes_read = copy_to_user(user_buffer, device_buffer + *offset, length);

    if (bytes_read != 0) {
        return -EFAULT;
    }

    //updates positions
    *offset += length;
    return length;
}

// write: writes the specified bytes starting at the current offset 
static ssize_t dev_write(struct file *fptr, const char *user_buffer, size_t length, loff_t *offset) {
    //checks that user params are within buffer size
    if ((*offset > BUFFER_SIZE) || ((*offset + length) > BUFFER_SIZE)) {
        return -EINVAL;
    }
    
    //writes bytes from user to kernel space
    size_t bytes_written = copy_from_user(device_buffer + *offset, user_buffer, length);

    if (bytes_written != 0) {
        return -EFAULT;
    }

    //updates positions
    *offset += length;
    buffer_size += length;

    return length;
}

// llseek:  sets the offset within the memory region (where the next read/write will start)
static loff_t dev_llseek(struct file *filp, loff_t offset, int whence) {
    loff_t new_pos = 0;

    //change position of file pointer based on user params
    switch (whence) {
        case SEEK_SET:
            new_pos = offset;
            break;
        case SEEK_CUR:
            new_pos = offset + filp->f_pos;
            break;
        case SEEK_END:
            new_pos = buffer_size + offset;
            break;
        default:
            return -EINVAL;
    }

    if ((new_pos < 0) || (new_pos > BUFFER_SIZE))  {
        return -EINVAL;
    }
    
    filp->f_pos = new_pos;
    return new_pos;
}

// release:closes the file descriptor (the default behavior) check
static int dev_release(struct inode *inode, struct file *file) {
    return 0;
}

// open: opens the device file and returns 0 if success check
static int dev_open(struct inode *inode, struct file *file) {
    return 0;
}

module_init(mymem_init);
module_exit(mymem_exit);