// lib.rs
use winit::window::Window;
use winit::event::WindowEvent;
use std::sync::Arc;
use std::borrow::Cow;
use wgpu::util::initialize_adapter_from_env_or_default;
use wgpu::Color;
use wgpu::util::DeviceExt;


pub mod buffer;

use buffer::{VERTICES, Vertex};

pub struct InitializedState {
    window: Arc<Window>,
    _instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    _adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    _shader: wgpu::ShaderModule,
    _pipeline_layout: wgpu::PipelineLayout,
    render_pipeline: wgpu::RenderPipeline,
    config: wgpu::SurfaceConfiguration,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
}


impl InitializedState {
    // Creating some of the wgpu types requires async code
    // ...
    pub async fn new(window: Arc<Window>) -> InitializedState {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::default();
        
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = initialize_adapter_from_env_or_default(&instance,Some(&surface))
                .await
                .expect("Failed to find a proper adapter!");
        
        let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        required_features: wgpu::Features::empty(),
                        // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                        required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                            .using_resolution(adapter.limits()),
                        memory_hints: Default::default(),
                    },
                    None,
                )
                .await
                .expect("Failed to create device");
        let config = surface
                .get_default_config(&adapter, size.width, size.height)
                .unwrap();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../res/shader.wgsl"))),
                });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                push_constant_ranges: &[],
                });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[
                        Vertex::desc(),
                    ],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    compilation_options: Default::default(),
                    targets: &[Some(swapchain_format.into())],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });
        
        let vertex_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(VERTICES),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );
        let num_vertices = VERTICES.len() as u32;

        surface.configure(&device, &config);
        
        

        Self {   
            window,
            _instance: instance,
            surface,
            _adapter: adapter,
            device,
            queue,
            config,
            _shader: shader,
            _pipeline_layout: pipeline_layout,
            render_pipeline,
            vertex_buffer,
            num_vertices
        }
    }


    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }    
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        todo!()
    }

    fn update(&mut self) {
        todo!()
    }

    pub fn request_redraw(&mut self) -> () {
        self.window.request_redraw();
    }

    pub fn render(&mut self, clear: Color) -> () {
        let frame = self
                    .surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
        let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.draw(0..self.num_vertices, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
