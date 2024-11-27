use opencv::core::{flip, Vec3b};
use opencv::videoio::*;
use opencv::{prelude::*, videoio, highgui::*};
use std::net::TcpStream;
use std::io::{Write, Read};
mod utils;
use utils::*;

struct App {
    cam: VideoCapture,
    server: ServerFacing,
}

impl App {
    fn new() -> Self {
        let mut cam = VideoCapture::new(0, videoio::CAP_ANY).unwrap();
        cam.set(CAP_PROP_FPS, 30.0).expect("Set camera FPS [FAILED]");
        let server = ServerFacing::new("10.0.2.2:2224");

        Self { cam, server }
    }

    fn run(&mut self) {
        loop {
            let mut frame = Mat::default();
            self.cam.read(&mut frame).expect("VideoCapture: read [FAILED]");

            if frame.size().unwrap().width > 0 {
                let mut flipped = Mat::default();
                flip(&frame, &mut flipped, 1).expect("flip [FAILED]");
                let resized_img = resize_with_padding(&flipped, [192, 192]);

                let vec_2d: Vec<Vec<Vec3b>> = resized_img.to_vec_2d().unwrap();
                let vec_1d: Vec<u8> = vec_2d.iter().flat_map(|v| v.iter().flat_map(|w| w.as_slice())).cloned().collect();
                let results = self.server.send_image(&vec_1d);

                draw_keypoints(&mut flipped, &results, 0.25);
                imshow("MoveNet", &flipped).expect("imshow [ERROR]");
            }

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

    fn send_image(&mut self, image: &[u8]) -> Vec<f32> {
        let len = (image.len() as u32).to_be_bytes();
        println!("Sending image data of length: {}", image.len());
        self.stream.write_all(&len).expect("Failed to send length");
        self.stream.write_all(image).expect("Failed to send image");

        let mut buffer = [0u8; 1024]; // Adjust buffer size as needed
        let n = self.stream.read(&mut buffer).expect("Failed to read from server");
        let results: Vec<f32> = bincode::deserialize(&buffer[..n]).expect("Failed to deserialize results");
        
        results
    }
}


fn main() {
    let mut app = App::new();
    app.run();
}
