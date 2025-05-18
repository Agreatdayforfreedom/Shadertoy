use crate::{
    quad::{Quad, VERTICES},
    texture::Texture,
};
use wgpu::util::DeviceExt;

#[allow(dead_code)]
pub struct Sprite {
    pub texture: Texture,
    pub bind_group: wgpu::BindGroup,
    pub buffer: wgpu::Buffer,
}

impl Sprite {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        am: wgpu::AddressMode,
        layout: &wgpu::BindGroupLayout,
        bytes: &[u8],
    ) -> Self {
        let texture = Texture::from_bytes(&device, &queue, bytes, am, "spaceship").unwrap();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            texture,
            bind_group,
            buffer,
        }
    }

    pub fn from_empty(
        device: &wgpu::Device,
        dimensions: (u32, u32),
        am: wgpu::AddressMode,
        layout: &wgpu::BindGroupLayout,
        label: &str,
    ) -> Self {
        let texture = Texture::empty(&device, dimensions, Some(am), Some(label)).unwrap();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            texture,
            bind_group,
            buffer,
        }
    }

    pub fn bind<'a, 'b>(&self, rpass: &'b mut wgpu::RenderPass<'a>) {
        rpass.set_vertex_buffer(0, self.buffer.slice(..));
        rpass.set_bind_group(1, &self.bind_group, &[]);
    }
}

pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some("texture_bind_group_layout"),
    })
}
