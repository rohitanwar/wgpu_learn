use wgpu::Color;
use winit::{
    application::ApplicationHandler, dpi::{LogicalSize, PhysicalSize}, event::WindowEvent, event_loop::ActiveEventLoop, window::{Window, WindowId}
};

use std::sync::Arc;

use crate::{pixel_pos_to_index, PixelBuffer};
use crate::renderer::InitializedState;
use crate::physical_to_pixel_pos;

//#[derive(Default)]
pub struct App {
    state: Option<InitializedState>,
    clear: Option<Color>,
    size: Option<PhysicalSize<u32>>,
    pixel_buffer: Arc<PixelBuffer>,
}

impl App {
    pub fn new(pixel_buffer: Arc<PixelBuffer>) -> App {
        Self {
            state:None,
            clear: None,
            size: None,
            pixel_buffer
        }
    }
}

impl ApplicationHandler for App {

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            #[allow(unused_mut)]
                let mut attributes = Window::default_attributes().with_max_inner_size(LogicalSize::new(800, 600));
            #[cfg(target_arch = "wasm32")]
            {
                log::info!("Creating canvas element for wasm32 target");
                use wasm_bindgen::JsCast;
                use winit::platform::web::WindowAttributesExtWebSys;
                let canvas = web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .get_element_by_id("canvas")
                    .unwrap()
                    .dyn_into::<web_sys::HtmlCanvasElement>()
                    .unwrap();
                attributes = attributes.with_canvas(Some(canvas));
            }

            // Move this into the state struct?
            let window = Arc::new(event_loop.create_window(attributes).unwrap());
            self.size = Some(window.inner_size());
            self.clear = Some(wgpu::Color::GREEN);
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.state = Some(pollster::block_on(InitializedState::new(window, self.pixel_buffer.clone())));
            }
            #[cfg(target_arch = "wasm32")]
            {
                wasm_bindgen_futures::spawn_local(async move {
                    let initialized_state = InitializedLoopState::new(window).await;
                    MESSAGE_QUEUE.with_borrow_mut(|queue| {
                        queue.push(Message::Initialized(initialized_state))
                    });
                });

            }
        }
    }


    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(state) = self.state.as_mut().as_mut() {
            match event {
                WindowEvent::Resized(new_size) => {
                    self.size = Some(new_size);
                    state.resize(new_size);
                    state.window().request_redraw();
                }
                WindowEvent::RedrawRequested => {
                    state.render(self.clear.unwrap(), self.pixel_buffer.clone());
                    state.request_redraw();
                }
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::CursorMoved { device_id: _, position } => {
                    let pixel_buffer: &mut PixelBuffer = Arc::get_mut(&mut self.pixel_buffer).unwrap();
                    let pixel_pos = physical_to_pixel_pos(position, self.size.unwrap(), pixel_buffer.size);
                    match pixel_pos{
                        Some(x) => {
                            let index = pixel_pos_to_index(x,pixel_buffer.size);
                            let pixels = &mut pixel_buffer.pixels;
                            pixels[index] = 0;
                            pixels[index + 1] = 255;
                            pixels[index + 2] = 255;
                        },
                        None => return,
                    }
                }
                _ => {}
            }
        }
    }

}