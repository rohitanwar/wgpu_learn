// lib.rs
use winit::window::Window;
use winit::event::WindowEvent;
use std::sync::Arc;
use std::borrow::Cow;
use wgpu::util::initialize_adapter_from_env_or_default;
use wgpu::Color;
use wgpu::util::DeviceExt;

use crate::texture;
use crate::buffer::{VERTICES, Vertex, INDICES};
use crate::PixelBuffer;

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
    _num_vertices: u32,
    index_buffer: wgpu::Buffer, 
    num_indices: u32,
    diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: texture::Texture
}


impl InitializedState {
    // Creating some of the wgpu types requires async code
    // ...
    pub async fn new(window: Arc<Window>, pixel_buffer: Arc<PixelBuffer>) -> InitializedState {
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
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../res/shader.wgsl"))),
                });

        surface.configure(&device, &config);


        //let diffuse_bytes = include_bytes!("../../res/happy-tree.png");
        
        let diffuse_texture = texture::Texture::from_pixel_buffer(&device, &queue, pixel_buffer, None);
        

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });
        
        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
            );

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&texture_bind_group_layout],
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
        
        let index_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(INDICES),
                    usage: wgpu::BufferUsages::INDEX,
                }
            );
            
        
        let num_vertices = VERTICES.len() as u32;
        let num_indices = INDICES.len() as u32;

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
            _num_vertices: num_vertices,
            index_buffer,
            num_indices,
            diffuse_bind_group, 
            diffuse_texture,
        }
    }


    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            println!("{} {}",new_size.width,new_size.height);
            self.surface.configure(&self.device, &self.config);
        }    
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        todo!()
    }

    fn update(&mut self) {
        todo!()
    }

    pub fn request_redraw(&mut self) -> () {
        self.window.request_redraw();
    }

    pub fn render(&mut self, clear: Color, pixels: Arc<PixelBuffer>) -> () {
        let size = wgpu::Extent3d {
            width: pixels.size.width,
            height: pixels.size.height,
            depth_or_array_layers: 1,
        };
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &self.diffuse_texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &pixels.pixels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * pixels.size.width),
                rows_per_image: Some(pixels.size.height),
            },
            size,
        );
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
            rpass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            rpass.draw_indexed(0..self.num_indices,0, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
