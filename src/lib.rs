use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, window::{Window, WindowId}
};

use wgpu::Color;

use std::sync::Arc;

mod renderer;
use crate::renderer::buffer;

use renderer::InitializedState;

#[derive(Default)]
struct App {
    state: Option<InitializedState>,
    clear: Option<Color>,
    size: Option<PhysicalSize<u32>>
}

// fn create_attribute(height:u32, width:u32, title:String) -> WindowAttributes {
//     Window::default_attributes().with_inner_size(LogicalSize::new(height,width)).with_title(title)
// }

// impl<'a> App<'a> {
//     async fn create_state(&mut self) -> () {
//         let window = self.window.as_ref().unwrap();
//         self.state = Some(State::new(window).await);
//     }
// }
impl ApplicationHandler for App {

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            #[allow(unused_mut)]
                let mut attributes = Window::default_attributes();
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
                self.state = Some(pollster::block_on(InitializedState::new(window)));
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
                    state.render(self.clear.unwrap());
                    state.request_redraw();
                }
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::CursorMoved { device_id, position } => {
                    let rx = (position.x)/(self.size.unwrap().width as f64);
                    let ry = (position.y)/(self.size.unwrap().height as f64);
                    self.clear = Some(wgpu::Color {
                        r : rx,
                        g : ry,
                        b : 1.0,
                        a : 1.0
                    });
                }
                _ => {}
            }
        }
    }


}

pub fn run() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app: App = App::default();
    let _ = event_loop.run_app(&mut app);
}

