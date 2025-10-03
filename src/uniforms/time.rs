#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TimeUniform {
    pub time: f32,
    _pad: f32,
    pub resulotion: [f32; 2],
}

impl Default for TimeUniform {
    fn default() -> Self {
        Self { time: 0.0, resulotion: [10.0, 10.0], _pad: 10.0 }
    }
}
