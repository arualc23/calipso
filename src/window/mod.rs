use egui_wgpu::wgpu;
use egui_winit::winit::{self, event};
use std::{sync::atomic::{AtomicBool, Ordering}};

mod wgpu_state;
mod render;
pub mod state;
use wgpu_state::WgpuState;
use state::State;

pub(crate) use render::EguiRenderer;

pub(crate) static CLOSING_REQUESTED: AtomicBool = AtomicBool::new(false);

#[derive(Default)]
pub(crate) struct ControlFlow<F: FnOnce(&render::EguiRenderer) -> Box<dyn State>> {
    state: Option<Box<dyn State>>,
    closing_requested: bool,
    wgpu_state: Option<WgpuState>,
    title: String,
    initializer: Option<F>,
}

impl<F: FnOnce(&render::EguiRenderer) -> Box<dyn State>> ControlFlow<F> {

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        println!("Resize called");
        let wgpu = self.wgpu_state.as_mut().unwrap();
        if new_size.width > 0 && new_size.height > 0 {
            wgpu.size = new_size;
            wgpu.config.width = new_size.width;
            wgpu.config.height = new_size.height;
            wgpu.surface.configure(&wgpu.device, &wgpu.config);
        }
    }

    fn render(&mut self) -> Result<bool, wgpu::CurrentSurfaceTexture> {
        let window = self.wgpu_state.as_ref().unwrap().window.clone();
        let wgpu = self.wgpu_state.as_mut().unwrap();
        let texture = wgpu.surface.get_current_texture();
        let output = match texture {
            wgpu::CurrentSurfaceTexture::Success(output) => output,
            wgpu::CurrentSurfaceTexture::Suboptimal(output) => output,
            other => return Err(other),
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

        let screen_desriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [wgpu.config.width, wgpu.config.height],
            pixels_per_point: window.scale_factor() as f32,
        };

        wgpu.egui.draw(
            &wgpu.device,
            &wgpu.queue,
            &window,
            &view,
            output,
            screen_desriptor,
            |ui| self.state.as_mut().unwrap().run_frame(ui)
        );

        self.check_transition_state();


        Ok(self.closing_requested)

    }

    fn check_transition_state(&mut self) {
        if let Some(state) = self.state.as_mut().unwrap().transition() {
            self.state = Some(state);
        }
    }

    fn close(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        CLOSING_REQUESTED.store(true, Ordering::Release);
        event_loop.exit();
    }

    pub fn new(initializer: F, title: String) -> Self {
        Self {
            state: None,
            closing_requested: false,
            wgpu_state: None,
            title,
            initializer: Some(initializer),
        }
    }
}

impl<F: FnOnce(&EguiRenderer) -> Box<dyn State>> winit::application::ApplicationHandler for ControlFlow<F> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {

        if self.wgpu_state.is_none() {
            self.wgpu_state = Some(pollster::block_on(WgpuState::new(event_loop, &self.title)));
        }

        if self.state.is_none() {
            let state = (self.initializer.take().expect("Cannot initialize the window twice"))(&self.wgpu_state.as_ref().unwrap().egui);
            self.state = Some(state);

        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: event::WindowEvent,
    )
    {
        let wgpu = self.wgpu_state.as_mut().unwrap();
        let response = wgpu.egui.handle_input(&wgpu.window, &event);

        if response.consumed {
            return;
        }

        match event {
            event::WindowEvent::CloseRequested  => self.close(&event_loop),
            event::WindowEvent::Resized(physical_size) => {
                self.resize(physical_size);
            }
            event::WindowEvent::RedrawRequested => {

                match self.render() {
                    Ok(closing_requested) => {
                        if closing_requested {
                            println!("Manual exit called... ");
                            self.close(&event_loop);
                            return;
                        }
                    }
                    Err(wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated) => {
                        self.resize(self.wgpu_state.as_ref().unwrap().size)
                    }
                    Err(wgpu::CurrentSurfaceTexture::Timeout) => log::warn!("Surface timeout"),
                    Err(wgpu::CurrentSurfaceTexture::Occluded) => (),
                    Err(wgpu::CurrentSurfaceTexture::Success(_)) | Err(wgpu::CurrentSurfaceTexture::Suboptimal(_)) => unreachable!("wtf"),
                    Err(wgpu::CurrentSurfaceTexture::Validation) => todo!("Validation error"),
                }
            }
            event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.wgpu_state.as_ref().unwrap().egui.ctx().set_pixels_per_point(scale_factor as f32);
            }
            _ => ()
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.wgpu_state.as_ref().unwrap().window.request_redraw();
    }
}