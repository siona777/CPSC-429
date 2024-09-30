use std::sync::{Arc, Mutex};
use std::thread;
use std::fs::OpenOptions;
use std::io::{Read, Write, Seek, SeekFrom};

const FILE_PATH: &str = "/dev/mymem"; // Modify this path to your file
const INIT_VAL: u64 = 0;
const NUM_THREADS: usize = 4; // Number of worker threads
const NUM_INCREMENTS: usize = 10000; // Number of increments each thread performs

fn main() {
    // Open the file and write the initial value
    initialize_file();

    // Create an Arc and Mutex for thread-safe access to the file
    let file_mutex = Arc::new(Mutex::new(
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(FILE_PATH)
            .expect("Failed to open file"),
    ));

    // Spawn worker threads
    let mut handles = vec![];
    for thread_id in 0..NUM_THREADS {
        let file_mutex = Arc::clone(&file_mutex);
        
        let handle = thread::spawn(move || {
            for _ in 0..NUM_INCREMENTS {
                let mut file = file_mutex.lock().unwrap();

                // Read the current value
                let mut buffer = [0u8; 8];
                file.seek(SeekFrom::Start(0)).expect("Failed to seek to the start of the file");
                file.read_exact(&mut buffer).expect("Failed to read value");
                let mut value = u64::from_ne_bytes(buffer);
                
                // Increment the value
                value += 1;
                println!("[Thread {}] Incremented value: {}", thread_id, value);
                
                // Write the new value back to the file
                let new_bytes = value.to_ne_bytes();
                file.seek(SeekFrom::Start(0)).expect("Failed to seek to the start of the file");
                file.write_all(&new_bytes).expect("Failed to write value");
                file.flush().expect("Failed to flush writes to the file");
            }
        });
        
        handles.push(handle);
    }

    // Wait for all threads to finish
    for handle in handles {
        handle.join().expect("Failed to join thread");
    }

    // Read and print the final value
    let mut file = OpenOptions::new()
        .read(true)
        .open(FILE_PATH)
        .expect("Failed to open file");
    
    let mut buffer = [0u8; 8];
    file.seek(SeekFrom::Start(0)).expect("Failed to seek to the start of the file");
    file.read_exact(&mut buffer).expect("Failed to read final value");
    let final_value = u64::from_ne_bytes(buffer);
    println!("Final value: {}", final_value);
}

fn initialize_file() {
    // Open the file for writing and initialize with INIT_VAL
    let mut file = OpenOptions::new()
        .write(true)
        .create(true) // Create the file if it does not exist
        .open(FILE_PATH)
        .expect("Failed to open file");

    let init_bytes = INIT_VAL.to_ne_bytes();
    file.write_all(&init_bytes).expect("Failed to write initial value");
    file.flush().expect("Failed to flush initial value");
    println!("Successfully initialized the file with value: {}", INIT_VAL);
}
