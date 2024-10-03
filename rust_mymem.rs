use kernel::prelude::*;
use kernel::sync::Mutex;

module! {
    type: RustMemoryDriver,
    name: "rust_memory_driver",
    author: "Your Name",
    description: "A simple memory driver implemented in Rust",
    license: "GPL",
}

const BUFFER_SIZE: usize = 1024;

struct MemoryBuffer {
    data: Mutex<[u8; BUFFER_SIZE]>,
}

impl MemoryBuffer {
    const fn new() -> Self {
        Self {
            data: unsafe {
                Mutex::new([0; BUFFER_SIZE])
            }
        }
    }
}

static MEMORY_BUFFER: MemoryBuffer = MemoryBuffer::new();

pub struct RustMemoryDriver;

impl kernel::Module for RustMemoryDriver {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust memory driver loaded\n");
        Ok(RustMemoryDriver)
    }
}

impl Drop for RustMemoryDriver {
    fn drop(&mut self) {
        pr_info!("Rust memory driver unloaded\n");
    }
}

impl RustMemoryDriver {
    // Read data from the memory driver's memory into outbuf
    pub fn read(&mut self, outbuf: &mut [u8], offset: usize) -> usize {
        let buffer = MEMORY_BUFFER.data.lock();
        let available = BUFFER_SIZE.saturating_sub(offset);
        let count = outbuf.len().min(available);
        outbuf[..count].copy_from_slice(&buffer[offset..offset + count]);
        count
    }

    // Write data into the memory driver's memory from inbuf
    pub fn write(&mut self, inbuf: &[u8], offset: usize) -> usize {
        let mut buffer = MEMORY_BUFFER.data.lock();
        let available = BUFFER_SIZE.saturating_sub(offset);
        let count = inbuf.len().min(available);
        buffer[offset..offset + count].copy_from_slice(&inbuf[..count]);
        count
    }
}