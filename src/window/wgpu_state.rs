use egui_wgpu::wgpu;
use egui_winit::winit::{self, window::Window};
use std::sync::Arc;
use super::render::EguiRenderer;

pub(crate) struct WgpuState {
    pub(crate) window: Arc<Window>,
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) size: winit::dpi::PhysicalSize<u32>,
    pub(crate) egui: EguiRenderer,
}

impl WgpuState {
    pub async fn new(event_loop: &winit::event_loop::ActiveEventLoop, title: &str) -> Self {
        let window = Arc::new(
        event_loop
            .create_window(Window::default_attributes().with_title(title))
            .unwrap(),
        );

        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            flags: wgpu::InstanceFlags::empty(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: Some(Box::new(event_loop.owned_display_handle())),
        });
        
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false
            })
            .await
            .unwrap();

        let adapter_limits = adapter.limits();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_limits: adapter_limits,
                    ..Default::default()
                }
                
            )
            .await
            .unwrap();
        
        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo, // force vsync, known good
            alpha_mode: wgpu::CompositeAlphaMode::Opaque, // force opaque
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        println!("adapter: {:?}", adapter.get_info());
        println!("surface_format: {:?}", config.format);
        println!("alpha_modes: {:?}", surface_caps.alpha_modes);
        println!("present_modes: {:?}", surface_caps.present_modes);

        println!("Surface format: {:?}", config.format);

        let mut egui = EguiRenderer::new(&device, 
        config.format,
            &window
        );

        egui.ctx().set_pixels_per_point(window.scale_factor() as f32);

        let screen_desriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [config.width, config.height],
            pixels_per_point: window.scale_factor() as f32,
        };

        let texture = surface.get_current_texture();
        let output = match texture {
            wgpu::CurrentSurfaceTexture::Success(output) => output,
            wgpu::CurrentSurfaceTexture::Suboptimal(output) => output,
            _ => panic!("First frame"),
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: None,
            dimension: None,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
            usage: None,
        });

        egui.draw(&device, &queue, &window, &view, output, screen_desriptor, |_| {});


        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            egui
        }

    }
}