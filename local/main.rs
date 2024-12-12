use std::io::{Read, Write};
use std::net::TcpStream;
use v4l::buffer::Type;
use v4l::io::traits::CaptureStream; // Import the CaptureStream trait
use v4l::prelude::*;
use v4l::video::capture::Parameters;
use v4l::io::mmap::Stream;
use v4l::video::Capture;
use v4l::{Format, FourCC};
use opencv::core::{Mat, Size, Scalar, CV_8UC3, Mat_AUTO_STEP};
use opencv::videoio::*;
use opencv::{prelude::*, videoio, highgui::*};
use opencv::imgproc;
use opencv::core::{flip, Vec3b};
use tflitec::interpreter::{Interpreter, Options};
use tflitec::model::Model;
mod utils;
use utils::*;


fn vec_to_mat(rgb_vec: Vec<Vec<(u8, u8, u8)>>) -> Mat {
    let height = rgb_vec.len();
    let width = if height > 0 { rgb_vec[0].len() } else { 0 };

    let mut flat_rgb_vec: Vec<u8> = Vec::with_capacity(height * width * 3);

    for row in rgb_vec {
        for (r, g, b) in row {
            flat_rgb_vec.push(r);
            flat_rgb_vec.push(g);
            flat_rgb_vec.push(b);
        }
    }

    let mat = Mat::from_slice(&flat_rgb_vec).expect("REASON").reshape(3, height as i32);
    mat.expect("REASON")
}

pub fn rgb_to_bgr_matrix(buffer: Vec<u8>, width: usize, height: usize) -> Mat {
    let mut matrix = vec![vec![(0, 0, 0); width]; height];
    let mut i = 0;

    for y in 0..height {
        for x in 0..width {
            let r = buffer[i];
            let g = buffer[i + 1];
            let b = buffer[i + 2];
            matrix[y][x] = (b, g, r); // Swap red and blue channels
            i += 3;
        }
    }

    let mat = vec_to_mat(matrix);
    mat
}

struct App {
    cam: Device,
    stream: MmapStream<'static>,
    server: ServerFacing,
}

impl App {
    fn new() -> Self {
        // Open video device
        println!("opening device");
        let cam = Device::new(0).expect("Failed to open video device");

        // Set camera parameters (resolution, format, etc.)
        println!("setting format");
        let format = Format::new(1280, 720, FourCC::new(b"YUYV"));
        cam.set_format(&format).expect("Failed to set format");

        // Initialize capture parameters with 30 FPS
        println!("params");
        let parameters = Parameters::with_fps(30);
        cam.set_params(&parameters).expect("Failed to set parameters");

        // Create a memory-mapped stream for capturing frames
        println!("stream");
        let stream = MmapStream::new(&cam, Type::VideoCapture).expect("Failed to create capture stream");

        // Initialize the server
        println!("server");
        let server = ServerFacing::new("10.0.2.2:2224");

        Self { cam, stream, server }
    }

    fn run(&mut self) {
        loop {
            // Capture a frame
            let (buf, _) = self.stream.next().expect("Failed to capture frame");
            
            // Send the image data to the server
            let (keypoints, rgb_image): (Vec<f32>, Vec<u8>) = self.server.send_image(&buf);

            let mut results_rgb_matrix = rgb_to_bgr_matrix(rgb_image, 1280, 720);

            draw_keypoints(&mut results_rgb_matrix, &keypoints, 0.25);

            imshow("Processed Image", &results_rgb_matrix).expect("Failed to display image");

            let key = wait_key(1).unwrap();
            if key > 0 && key != 255 {
                break;
            }
        }
    }
}



struct ServerFacing {
    stream: TcpStream,
}

impl ServerFacing {
    fn new(server_addr: &str) -> Self {
        let stream = TcpStream::connect(server_addr).expect("Could not connect to server");
        Self { stream }
    }

    fn send_image(&mut self, image: &[u8]) -> (Vec<f32>, Vec<u8>) {
        let len = (image.len() as u32).to_be_bytes();
        println!("Sending image data of length: {}", image.len());
        self.stream.write_all(&len).expect("Failed to send length");
        self.stream.write_all(image).expect("Failed to send image");
    
        // Receive the length of the incoming data (results + RGB image)
        let mut length_buffer = [0u8; 4];
        self.stream.read_exact(&mut length_buffer).expect("Failed to read length");
        let total_length = u32::from_be_bytes(length_buffer) as usize;
    
        println!("Expected receiving data length: {}", total_length); // Debugging print
    
        // Create a buffer to receive the data
        let mut received_buffer = vec![0u8; total_length];
        self.stream.read_exact(&mut received_buffer).expect("Failed to read from server");

        // Deserialize the results
        let result_size = bincode::serialized_size(&bincode::deserialize::<Vec<f32>>(&received_buffer).expect("Failed to deserialize results")).expect("Failed to get size of results") as usize;
        println!("Result serialized size: {}", result_size); // Debugging print

        let keypoints: Vec<f32> = bincode::deserialize(&received_buffer[..result_size]).expect("Failed to deserialize results");

        // Extract the remaining buffer as the RGB image
        let rgb_image = received_buffer[result_size..].to_vec();
        println!("RGB Image buffer size: {}", rgb_image.len()); // Debugging print
    
        (keypoints, rgb_image)
    }
}

fn main() {
    let mut app = App::new();
    app.run();
}
