use notify::Watcher;
use std::{path::Path, time::Duration};
use wgpu::naga;

use crate::{input_manager::{InputEvent, InputManager}, uniforms::uniforms::MainUniforms};
use std::sync::mpsc;

use crate::{
    quad::Quad,
    sprite::{create_bind_group_layout, Sprite},
    uniform::Uniform,
};



pub struct Stoy {
    test_sprite: Sprite,
    pipeline: wgpu::RenderPipeline,
    camera: Camera2D,
    uniforms: Uniform<MainUniforms>,
    channel: (mpsc::Sender<String>, mpsc::Receiver<String>),
    pipeline_layout: wgpu::PipelineLayout,
    read_lock: std::sync::Arc<std::sync::Mutex<Option<std::time::Instant>>>,
    input: InputManager,
    // unused - avoid dropping the watcher
    _watcher: notify::RecommendedWatcher,
}

impl Stoy {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: &wgpu::TextureFormat) -> Self {
        //uniforms
        let uniforms = Uniform::<MainUniforms>::new(device);
        let camera_uniform = Uniform::<Camera2DUniform>::new(device);
        //gruops
        let camera = Camera2D::new(camera_uniform);
        let sprite_layout = create_bind_group_layout(device);
        let bytes = include_bytes!("../assets/test.png");
        let test_sprite = Sprite::new(
            device,
            queue,
            wgpu::AddressMode::ClampToEdge,
            &sprite_layout,
            bytes,
        );

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Main_pipeline_layout"),
            bind_group_layouts: &[
                &camera.uniform.bind_group_layout,
                &sprite_layout,
                &uniforms.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("./shaders/sprite.wgsl"));
        let pipeline = create_render_pipeline(device, &shader, *format, &pipeline_layout);

        let (tx, rx) = std::sync::mpsc::channel::<String>();

        let read_lock = std::sync::Arc::new(std::sync::Mutex::new(None));
        let read_lock_clone = read_lock.clone();

        let mut watcher = notify::RecommendedWatcher::new(
            move |res: notify::Result<notify::Event>| match res {
                Ok(evt) => {
                    println!("evt: {:?}", evt);
                    if evt.kind.is_modify() || evt.kind.is_create() {
                        let mut t = read_lock_clone.lock().unwrap();
                        *t = Some(instant::Instant::now());
                        /*println!("Reading sprite.wgsl...");
                        }*/
                    }
                }
                Err(e) => eprintln!("watch error: {:?}", e),
            },
            notify::Config::default(),
        )
        .unwrap();

        watcher
            .watch(Path::new("./src/shaders/"), RecursiveMode::Recursive)
            .expect("Watcher throws!");

        Self {
            test_sprite,
            pipeline,
            uniforms,
            camera,
            channel: (tx, rx),
            pipeline_layout,
            input: InputManager::default(),
            read_lock,
            _watcher: watcher,
        }
    }

    pub fn input(&mut self, event: InputEvent) {
    
        self.input.process_events(event);

    }

    pub fn update(&mut self, queue: &mut wgpu::Queue, size: (u32, u32)) {
        self.uniforms.data.time += 0.01;
        self.uniforms.data.resulotion = [size.0 as f32, size.1 as f32];
        self.uniforms.data.mouse_position = [self.input.x as f32, self.input.y as f32];
        self.uniforms.data.zoom = [self.input.wx.abs(), self.input.wy.abs()];

        self.camera.uniform.write(queue);
        self.uniforms.write(queue);
    }
    pub fn render(
        &mut self,
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        queue: &mut wgpu::Queue,
        format: &wgpu::TextureFormat,
    ) {
        let mut g = self.read_lock.lock().unwrap();
        if let Some(last) = *g {
            if last.elapsed() > Duration::from_millis(200) {
                println!("Last: {}", last.elapsed().as_secs_f32());
                    if let Ok(src) = std::fs::read_to_string("./src/shaders/sprite.wgsl") {
                    let _ = self.channel.0.send(src);
                }
                *g = None;
            }
        }

        if let Ok(new_src) = self.channel.1.try_recv() {

            println!("Waiting... {} <<<", new_src);
            match naga::front::wgsl::parse_str(&new_src) {
                Ok(module) => {
                    let mut validator = naga::valid::Validator::new(
                        naga::valid::ValidationFlags::all(),
                        naga::valid::Capabilities::default(),
                    );
                    if let Err(validation_err) = validator.validate(&module) {
                        eprintln!("Hot-reload: WGSL validation error: {:?}", validation_err);
                        return;
                    }
                    match try_rebuild_pipeline(&device, &new_src, &self.pipeline_layout, *format) {
                        Ok(new_pipeline) => {
                            self.pipeline = new_pipeline;
                            eprintln!("Shader reloaded successfully!");
                        }
                        Err(err) => {
                            eprintln!("Shader reload failed:\n{}", err);
                        }
                    }
                }
                Err(parse_err) => {
                    eprintln!("Hot-reload: failed to parse WGSL: {:?}", parse_err);
                    return;
                }
            }
        }
        let frame = surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let context_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &context_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.camera.uniform.bind_group, &[]);
            self.test_sprite.bind(&mut rpass);
            rpass.set_bind_group(2, &self.uniforms.bind_group, &[]);

            rpass.draw(0..6, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));

        frame.present();
    }
}
fn try_rebuild_pipeline(
    device: &wgpu::Device,
    wgsl_source: &str,
    pipeline_layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
) -> Result<wgpu::RenderPipeline, String> {
    // Create shader module
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("hot_shader"),
        source: wgpu::ShaderSource::Wgsl(wgsl_source.into()),
    });
    let pipeline = create_render_pipeline(device, &shader, format, pipeline_layout);
    Ok(pipeline)
}
pub fn create_render_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
    pipeline_layout: &wgpu::PipelineLayout,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[Quad::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::Zero,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                }),
                write_mask: wgpu::ColorWrites::all(),
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}

use cgmath::{Matrix4, Vector2, Vector3};
use notify::RecursiveMode;

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - 0.01;
const SAFE_MIN_RADIUS: f32 = 1.0;
pub struct Camera2D {
    pub position: Vector3<f32>,
    pub scale: Vector2<f32>,
    pub uniform: Uniform<Camera2DUniform>,
}

impl Camera2D {
    pub fn new(uniform: Uniform<Camera2DUniform>) -> Self {
        Self {
            uniform,
            scale: (WIDTH, HEIGHT).into(),
            position: (0.0, 0.0, 0.0).into(),
        }
    }
    pub fn update(&mut self, position: Vector3<f32>) {
        self.position = position;
        self.uniform.data.update(position);
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera2DUniform {
    pub proj: [[f32; 4]; 4],
}
impl Camera2DUniform {
    fn update(&mut self, position: Vector3<f32>) {
        let view = Matrix4::from_translation(-position);
        let ortho = OPENGL_TO_WGPU_MATRIX
            * cgmath::ortho(
                -WIDTH / 2.0,
                WIDTH / 2.0,
                -HEIGHT / 2.0,
                HEIGHT / 2.0,
                -50.0,
                50.0,
            );
        self.proj = (ortho * view).into();
    }
}

impl Default for Camera2DUniform {
    fn default() -> Self {
        let position = Vector3::new(0.0, 0.0, 1.0);

        let view = Matrix4::from_translation(-position);
        let ortho = cgmath::ortho(0.0, 800.0, 600.0, 0.0, -50.0, 50.0);
        Self {
            proj: (ortho * view).into(),
        }
    }
}
