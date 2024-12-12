use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use tflitec::interpreter::{Interpreter, Options};
use tflitec::model::Model;
use opencv::core::{flip, Vec3b, Mat};
use opencv::videoio::*;
use opencv::{prelude::*, videoio, highgui::*};
mod utils;
use utils::*;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:2224").expect("Could not bind to port");

    for stream in listener.incoming() {
        let mut stream = stream.expect("Failed to accept connection");
        handle_client(&mut stream);
    }
}

fn handle_client(stream: &mut TcpStream) {
    let options = Options::default();
    let path = "resource/lite-model_movenet_singlepose_lightning_tflite_int8_4.tflite";
    let model = Model::new(&path).expect("Load model [FAILED]");
    let interpreter = Interpreter::new(&model, Some(options)).expect("Create interpreter [FAILED]");
    interpreter.allocate_tensors().expect("Allocate tensors [FAILED]");
    
    let width = 1280;
    let height = 720;

    loop {
        let mut length_buffer = [0u8; 4];
        stream.read_exact(&mut length_buffer).expect("Failed to read length");
        let length = u32::from_be_bytes(length_buffer) as usize;

        let mut image_buffer = vec![0u8; length];
        stream.read_exact(&mut image_buffer).expect("Failed to read image");

        let rgb_buf_from_yuv = yuv422_to_rgb(&image_buffer, width, height);

        let rgb_mat = rgb_to_matrix(rgb_buf_from_yuv.clone(), width, height);

        println!("rgb mat size {:?}", rgb_mat.size());

        let resized_img = resize_with_padding(&rgb_mat, [192, 192]);

        let vec_2d: Vec<Vec<Vec3b>> = resized_img.to_vec_2d().unwrap();
        let vec_1d: Vec<u8> = vec_2d.iter().flat_map(|v| v.iter().flat_map(|w| w.as_slice())).cloned().collect();

        interpreter.copy(&vec_1d, 0).unwrap();
        interpreter.invoke().expect("Invoke [FAILED]");

        let output_tensor = interpreter.output(0).unwrap();
        let results = output_tensor.data::<f32>().to_vec();
        let results = bincode::serialize(&results).expect("Failed to serialize results");

        let total_length = results.len() + rgb_buf_from_yuv.len();

        stream.write_all(&(total_length as u32).to_be_bytes()).expect("Failed to send length");

        // Send the serialized results
        stream.write_all(&results).expect("Failed to send results");

        // Send the serialized RGB image
        stream.write_all(&rgb_buf_from_yuv).expect("Failed to send RGB image");

    }
}

fn yuv422_to_rgb(yuv_buffer: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut rgb_buffer = vec![0u8; width * height * 3]; // Adjusted to 192x192 accordingly

    for y in 0..height {
        for x in 0..width / 2 {
            let i = (y * width / 2 + x) * 4;
            let y0 = yuv_buffer[i] as i32;
            let u = yuv_buffer[i + 1] as i32;
            let y1 = yuv_buffer[i + 2] as i32;
            let v = yuv_buffer[i + 3] as i32;

            let c0 = y0 - 16;
            let c1 = y1 - 16;
            let d = u - 128;
            let e = v - 128;

            let r0 = clip((298 * c0 + 409 * e + 128) >> 8);
            let g0 = clip((298 * c0 - 100 * d - 208 * e + 128) >> 8);
            let b0 = clip((298 * c0 + 516 * d + 128) >> 8);

            let r1 = clip((298 * c1 + 409 * e + 128) >> 8);
            let g1 = clip((298 * c1 - 100 * d - 208 * e + 128) >> 8);
            let b1 = clip((298 * c1 + 516 * d + 128) >> 8);

            rgb_buffer[(y * width + x * 2) * 3] = r0 as u8;
            rgb_buffer[(y * width + x * 2) * 3 + 1] = g0 as u8;
            rgb_buffer[(y * width + x * 2) * 3 + 2] = b0 as u8;
            rgb_buffer[(y * width + x * 2 + 1) * 3] = r1 as u8;
            rgb_buffer[(y * width + x * 2 + 1) * 3 + 1] = g1 as u8;
            rgb_buffer[(y * width + x * 2 + 1) * 3 + 2] = b1 as u8;
        }
    }

    rgb_buffer
}

fn clip(value: i32) -> i32 {
    if value < 0 {
        0
    } else if value > 255 {
        255
    } else {
        value
    }
}

pub fn rgb_to_matrix(buffer: Vec<u8>, width: usize, height: usize) -> Mat {
    let mut matrix = vec![vec![(0, 0, 0); width]; height];
    let mut i = 0;

    for y in 0..height {
        for x in 0..width {
            let r = buffer[i];
            let g = buffer[i + 1];
            let b = buffer[i + 2];
            matrix[y][x] = (r, g, b);
            i += 3;
        }
    }

    let mat = vec_to_mat(matrix);
    mat
}

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
