use std::{num::NonZeroU32, sync::Arc};

use bytemuck::Contiguous;
use wgpu::{SurfaceTexture, TextureFormat};
use winit::{
    dpi::PhysicalSize,
    event::*,
    window::{self, Window},
};

use crate::{input_manager::InputEvent, stoy::Stoy};

#[allow(dead_code)]
pub struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window: Arc<Window>,
    engine: Stoy,
}

impl GpuState {
    pub async fn new(window: Window) -> Self {
        let window = Arc::new(window);

        let size = window.inner_size();

        println!("w:{}, h: {}", size.width, size.height);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        println!("{:?}", adapter.features());

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    memory_hints: wgpu::MemoryHints::default(),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let limits = device.limits();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        // log::info!("surface caps: {:?}", &surface_caps);
        // log::info!("surface format: {:?}", &surface_format);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(800),
            height: size.height.max(600), // setting this because Fullscreen does not work on web: https://developer.mozilla.org/en-US/docs/Glossary/Transient_activation
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        let engine = Stoy::new(&device, &queue, &config.format);
        Self {
            surface,
            device,
            queue,
            config,
            window,
            engine,
        }
    }

    pub fn window(&self) -> &Window {
        self.window.as_ref()
    }

    pub fn input(&mut self, event: InputEvent) -> bool {
        self.engine.input(event);
        false
    }
    pub fn update(&mut self, dt: instant::Duration) {
//        println!("delta: {}", dt.as_secs_f32());
  //      println!("fps: {}", (1.0 / dt.as_secs_f32()));
        self.engine.update(&mut self.queue, (self.config.width, self.config.height));
    }
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        let (width, height) = match (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) {
            (Some(width), Some(height)) => (width, height),
            _ => return,
        };

        self.config.width = width.into();
        self.config.height = height.into();

        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self) {
        self.engine
            .render(&self.surface, &self.device, &mut self.queue, &self.config.format);
    }
}
