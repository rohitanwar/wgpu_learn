use wgpu::Color;
use winit::{dpi::{PhysicalPosition, PhysicalSize}, event_loop::{ControlFlow, EventLoop}};

pub mod renderer;
pub mod buffer;
pub mod texture;
pub mod app;

use app::App;
use std::sync::Arc;
pub struct PixelBuffer {
    pixels: Vec<u8>,
    size: PixelBufferSize
}

#[derive(Clone, Copy)]
pub struct PixelBufferSize {
    width: u32,
    height: u32
}
#[derive(Clone, Copy)]
pub struct PixelPosition {
    x: u32,
    y: u32
}

pub fn physical_to_pixel_pos(pos: PhysicalPosition<f64>, physical_size: PhysicalSize<u32>, pixel_buffer_size: PixelBufferSize) -> Option<PixelPosition>{
    let rx = pos.x.floor() as u32;
    let ry = pos.y.floor() as u32;

    let scalex = (physical_size.width/pixel_buffer_size.width).max(1);
    let scaley = (physical_size.height/pixel_buffer_size.height).max(1);

    let scale = scalex.max(scaley);

    let width = physical_size.width / scale;
    let height = physical_size.height / scale;


    let x = rx / scale + (pixel_buffer_size.width - width)/2;
    let y = ry / scale + (pixel_buffer_size.height - height)/2;
    if x > width || y > height {return None}

    assert!(x <= pixel_buffer_size.width);
    assert!(y <= pixel_buffer_size.height);

    return Some(PixelPosition{x,y});
}

pub fn pixel_pos_to_index(pos: PixelPosition, size: PixelBufferSize) -> usize {
    return ((pos.x + pos.y * size.width) * 4) as usize;
}

impl PixelBuffer {
    pub fn new(
        width: u32,
        height: u32,
        color: Color,
    ) -> PixelBuffer {
        let mut pixels = vec![0; (width * height * 4) as usize];
        for i in (0..pixels.len()).step_by(4) {
            pixels[i] = (color.r * 255.0).floor() as u8;
            pixels[i + 1] = (color.g * 255.0).floor() as u8;
            pixels[i + 2] = (color.b * 255.0).floor() as u8;
            pixels[i + 3] = (color.a * 255.0).floor() as u8;
        }
        Self {pixels, size: PixelBufferSize {width, height}}
    }

}

pub fn run() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    
    
    let pixels = Arc::new(PixelBuffer::new(50, 30, wgpu::Color::RED));
    let mut app: App = App::new(pixels);
    
    let _ = event_loop.run_app(&mut app);
}

