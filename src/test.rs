// use cgmath::Point2;

use crate::{
    quad::Quad,
    sprite::{create_bind_group_layout, Sprite},
    uniform::Uniform,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TestData {
    position: [f32; 2],
}

impl Default for TestData {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
        }
    }
}

pub struct Test {
    test_uniform: Uniform<TestData>,
    test_sprite: Sprite,
    test_pipeline: wgpu::RenderPipeline,
    camera: Camera2D,
}

impl Test {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: &wgpu::TextureFormat) -> Self {
        //uniform
        let test_uniform = Uniform::<TestData>::new(device);
        //gruops
        let camera_uniform = Uniform::<Camera2DUniform>::new(device);
        let camera = Camera2D::new(camera_uniform);
        // let test_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { entries });
        let sprite_layout = create_bind_group_layout(device);
        let bytes = include_bytes!("../assets/test.png");
        let test_sprite = Sprite::new(
            device,
            queue,
            wgpu::AddressMode::ClampToEdge,
            &sprite_layout,
            bytes,
        );

        //pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Main_pipeline_layout"),
            bind_group_layouts: &[&camera.uniform.bind_group_layout, &sprite_layout],
            push_constant_ranges: &[],
        });
        let shader = device.create_shader_module(wgpu::include_wgsl!("./shaders/sprite.wgsl"));
        let test_pipeline = create_render_pipeline(device, &shader, *format, &pipeline_layout);
        Self {
            test_uniform,
            test_sprite,
            test_pipeline,
            camera,
        }
    }

    pub fn update(&mut self, queue: &mut wgpu::Queue) {
        self.camera.uniform.write(queue);
    }
    pub fn render(
        &mut self,
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        queue: &mut wgpu::Queue,
    ) {
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

            //pipeline
            rpass.set_pipeline(&self.test_pipeline);

            rpass.set_bind_group(0, &self.camera.uniform.bind_group, &[]);

            self.test_sprite.bind(&mut rpass);
            rpass.draw(0..6, 0..1);
        }

        queue.submit(Some(encoder.finish()));

        frame.present();
    }
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
            buffers: &[
                Quad::desc(),
                // wgpu::VertexBufferLayout {
                //     array_stride: 8 * 4,
                //     step_mode: wgpu::VertexStepMode::Instance,
                //     attributes: &[
                //         //position
                //         wgpu::VertexAttribute {
                //             format: wgpu::VertexFormat::Float32x4,
                //             offset: 0,
                //             shader_location: 1,
                //         },
                //         wgpu::VertexAttribute {
                //             format: wgpu::VertexFormat::Float32x4,
                //             offset: 0,
                //             shader_location: 1,
                //         },
                //     ],
                // },
            ],
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
use wgpu::core::device::queue;

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
