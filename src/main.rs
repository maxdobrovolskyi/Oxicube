//winit
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowAttributes, WindowId};
use winit::dpi::{LogicalSize};
use winit::keyboard::KeyCode;
//tacing
use tracing::info;
//wgpu
use wgpu::{self};

//main
use std::sync::Arc;

struct GpuContext {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
}
struct App {
    window: Option<Arc<dyn Window>>,
    gpu: Option<GpuContext>
}

impl GpuContext {
    async fn new(window: Arc<dyn Window>) -> Self {
        let size = window.surface_size();

        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor::new_without_display_handle_from_env()
        );   

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase { 
                power_preference: wgpu::PowerPreference::HighPerformance, 
                force_fallback_adapter: false, 
                compatible_surface: Some(&surface), 
            })
            .await
            .expect("GPU not found");    

        println!("GPU: {:#?}", adapter.get_info().name);
        println!("Backend: {:#?}", adapter.get_info().backend);

        let (device, queue) = adapter
            .request_device(
            &wgpu::DeviceDescriptor {
                    label: Some("Main Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                    experimental_features: wgpu::ExperimentalFeatures::disabled(),
                    trace: wgpu::Trace::Off,
                },
            )
            .await
            .expect("Device failed");

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT, 
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo, 
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[],
            ..Default::default()
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            multiview_mask: Default::default(),
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],   
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
        }
    }
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}

impl ApplicationHandler for App {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let window = event_loop
            .create_window(WindowAttributes::default()
                .with_title("Oxicube")
                .with_surface_size(LogicalSize::new(1280.0, 720.0)))
                .unwrap();
        let window: Arc<dyn Window> = Arc::from(window);
        self.window = Some(window.clone());

        let gpu = pollster::block_on(GpuContext::new(window));
        self.gpu = Some(gpu)
    }
    fn window_event(
            &mut self,
            event_loop: &dyn ActiveEventLoop,
            window_id: WindowId,
            event: WindowEvent,
        ) {
        match event {
            WindowEvent::RedrawRequested => {
                if let Some(gpu) = &self.gpu {
                    let output = match gpu.surface.get_current_texture() {
                        wgpu::CurrentSurfaceTexture::Success(tex)
                        | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => tex,
                        _ => return, 
                    };
                    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

                    let mut encoder = gpu.device.create_command_encoder(
                        &wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") }
                    );

                    {
                        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Render Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                depth_slice: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                        r: 0.1, g: 0.1, b: 0.15, a: 1.0,
                                    }),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            ..Default::default()
                        });

                        render_pass.set_pipeline(&gpu.render_pipeline);  
                        render_pass.draw(0..3, 0..1);   
                    }
                    gpu.queue.submit(std::iter::once(encoder.finish()));
                    output.present();
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    } 
                }

                
            }
            WindowEvent::CloseRequested => {
                println!("Close window");
                event_loop.exit();
            }
            WindowEvent::SurfaceResized(size) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.resize(size);
                }
            }
            WindowEvent::KeyboardInput {event, .. } => {
                if let Some(text) = &event.text {
                    println!("key={:?} state={:?} text={:?}",
                        event.physical_key,
                        event.state,
                        text
                    );
                }
                if event.physical_key == PhysicalKey::Code(KeyCode::Escape) {
                    println!("Close window");
                    event_loop.exit();
                }
            }

            _ => {}
        }
    }
}

fn main() {
    //init logging
    tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(true)
        .pretty()
        .init();

    info!("App started");

    let event_loop = EventLoop::new().unwrap();
    let app = App { window: None , gpu: None};
    
    event_loop.run_app(Box::leak(Box::new(app))).unwrap();
}