//! mymemthread module in Rust.
use kernel::bindings::*;
use kernel::pr_cont;
use kernel::prelude::*;
use kernel::sync::{CondVar, Mutex};
use kernel::task::Task;
use kernel::Module;

module! {
    type: RustMemoryDriverTest,
    name: "rust_test",
    author: "Siona Tagare",
    description: "A Memory Driver Kernel Test Module",
    license: "GPL",
}

struct RustMemoryDriverTest;

fn nanoseconds() -> i64 {
    let mut t: timespec64 = timespec64 {
        tv_sec: 0,
        tv_nsec: 0,
    };

    unsafe {
        ktime_get_ts64(&mut t as *mut timespec64);
    }

    return t.tv_nsec;
}

const TRIALS: usize = 10;

fn measure(driver: &mut rust_memory_driver::RustMemoryDriver, size: usize, operation: &str) {
    let mut buffer = Vec::new();
    buffer.try_resize(size/8, 0);

    let mut total_time = 0;

    for _ in 0..TRIALS {
        let start = nanoseconds();
        match operation {
            "write" => {
                driver.write(&mut buffer, 0);
            }
            "read" => {
                driver.read(&mut buffer, 0);
            }
            _ => (),
        }
        let elapsed_time = nanoseconds() - start;
        total_time += elapsed_time;
    }

    pr_info!(
        "{} {} bytes took an average of {} nanoseconds",
        operation,
        size,
        total_time / TRIALS as i64
    );
}

fn multi_thread(n: usize, num_workers: u64) {
    // SPAWN DIFFERENT THREADS
    *COUNT.lock() = num_workers;
    for i in 0..num_workers {
        Task::spawn(fmt!("test{i}"), move || {
            let mut driver = rust_memory_driver::RustMemoryDriver;

            let mut guard = COUNT.lock();
            *guard -= 1;
            if *guard == 0 {
                COUNT_IS_ZERO.notify_all();
            }

            // PERFORM THE DATA RACE TEST
            let mut buf_64: [u8; 8] = [0; 8];

            for _ in 0..n {
                driver.read(&mut buf_64, 0);
                let mut val: u64 = u64::from_ne_bytes(buf_64[..].try_into().unwrap());
                val += 1;
                buf_64 = val.to_ne_bytes();
                driver.write(&buf_64, 0);
            }
        })
        .unwrap();
    }

    // WAIT FOR COUNT TO DROP TO ZERO
    let mut guard = COUNT.lock();
    while *guard != 0 {
        let _wait: bool = COUNT_IS_ZERO.wait(&mut guard);
    }
}

kernel::init_static_sync! {
    static COUNT: Mutex<u64> = 0;
    static COUNT_IS_ZERO: CondVar;
}

impl Module for RustMemoryDriverTest {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("<Part 3 Test>");
        let mut driver = rust_memory_driver::RustMemoryDriver;


        driver.write(b"00000000", 0);

        pr_info!("start: {}", u64::from_ne_bytes(*b"00000000"));
        multi_thread(1000, 10);

        let mut buf64: [u8; 8] = [0; 8];
        driver.read(&mut buf64, 0);
        let count = u64::from_ne_bytes(buf64[..].try_into().unwrap());

        pr_info!("end {}\n", count);

        pr_info!("Memory driver test module loaded\n");


        pr_info!("<Part 2 Test>");

        let sizes = [1, 64, 1024, 65536];
        for &size in &sizes {
            measure(&mut driver, size, "write");
            measure(&mut driver, size, "read");
        }


        Ok(RustMemoryDriverTest)
    }
}

impl Drop for RustMemoryDriverTest {
    fn drop(&mut self) {
        pr_info!("Memory driver test module unloaded\n");
    }
}
