use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use tflitec::interpreter::{Interpreter, Options};
use tflitec::model::Model;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:2224").expect("Could not bind to port");

    for stream in listener.incoming() {
        let mut stream = stream.expect("Failed to accept connection");
        handle_client(&mut stream);
    }
}

fn handle_client(stream: &mut TcpStream) {
    let options = Options::default();
    let path = format!("resource/lite-model_movenet_singlepose_lightning_tflite_int8_4.tflite");
    let model = Model::new(&path).expect("Load model [FAILED]");
    let interpreter = Interpreter::new(&model, Some(options)).expect("Create interpreter [FAILED]");
    interpreter.allocate_tensors().expect("Allocate tensors [FAILED]");
    
    loop {
        let mut length_buffer = [0u8; 4];
        stream.read_exact(&mut length_buffer).expect("Failed to read length");
        let length = u32::from_be_bytes(length_buffer) as usize;

        let mut image_buffer = vec![0u8; length];
        stream.read_exact(&mut image_buffer).expect("Failed to read image");

        interpreter.copy(&image_buffer, 0).unwrap();
        interpreter.invoke().expect("Invoke [FAILED]");

        let output_tensor = interpreter.output(0).unwrap();
        let results = output_tensor.data::<f32>().to_vec();
        let results = bincode::serialize(&results).expect("Failed to serialize results");

        stream.write_all(&results).expect("Failed to send results");
    }
}
