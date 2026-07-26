use egui_winit::{egui};

use egui::epaint::Shadow;
use egui::{Color32, Context, Visuals};
use egui_wgpu::ScreenDescriptor;
use egui_wgpu::Renderer;

use egui_winit::{EventResponse, State};
use egui_wgpu::wgpu::{Device, Queue, SurfaceTexture, TextureFormat, TextureView};
use egui_wgpu::wgpu;
use egui_winit::winit::event::WindowEvent;
use egui_winit::winit::window::Window;

use std::iter;

pub struct EguiRenderer {
    state: State,
    renderer: Renderer,
}

impl EguiRenderer {
    pub fn ctx(&self) -> &egui::Context {
        &self.state.egui_ctx()
    }
    pub fn new(
        device: &Device,
        output_color_format: TextureFormat,
        window: &Window,
    ) -> EguiRenderer {
        let egui_context = Context::default();
        let id = egui_context.viewport_id();

        egui_context.set_pixels_per_point(window.scale_factor() as f32);
        egui_context.set_fonts(egui::FontDefinitions::default());

        let visuals = Visuals {
            window_shadow: Shadow::NONE,
            panel_fill: Color32::from_hex("#1e3837").unwrap(),
            ..Default::default()
        };

        egui_context.set_visuals(visuals);

        let egui_state = State::new(
            egui_context, 
            id, 
            &window, 
            None, 
            None,
            None
        );

        let egui_renderer = Renderer::new(
            device,
            output_color_format,
            egui_wgpu::RendererOptions::default(),
        );

        EguiRenderer {
            state: egui_state,
            renderer: egui_renderer,
        }
    }

    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) -> EventResponse {
        self.state.on_window_event(window, event)
    }

    pub fn draw(
        &mut self,
        device: &Device,
        queue: &Queue,
        window: &Window,
        window_surface_view: &TextureView,
        surface_texture: SurfaceTexture,
        screen_descriptor: ScreenDescriptor,
        mut run_ui: impl FnMut(&Context),
    ) {

        let mut raw_input = self.state.take_egui_input(&window);
        let max = device.limits().max_texture_dimension_2d as usize;

        raw_input.max_texture_side = Some(max);

        let full_output = self.ctx().run_ui(raw_input, |ui| {
            run_ui(ui);
        });

        self.state
            .handle_platform_output(&window, full_output.platform_output);


        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(&device, &queue, *id, &image_delta);
        }

        let tris = self.ctx().tessellate(full_output.shapes, full_output.pixels_per_point);

        let mut encoder = device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        
        self.renderer
            .update_buffers(&device, &queue, &mut encoder, &tris, &screen_descriptor);
        let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &window_surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None
            })],
            depth_stencil_attachment: None,
            label: Some("egui main render pass"),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        let mut rpass = rpass.forget_lifetime();

        self.renderer.render(&mut rpass, &tris, &screen_descriptor);
        drop(rpass);

        queue.submit(iter::once(encoder.finish()));
        surface_texture.present();

        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }
    }
}
