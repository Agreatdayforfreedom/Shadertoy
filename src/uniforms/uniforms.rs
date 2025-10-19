#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MainUniforms {
    pub time: f32,
    _pad: f32,
    pub resulotion: [f32; 2],
    pub mouse_position: [f32; 2],
    pub zoom: [f32; 2],
}

impl Default for MainUniforms {
    fn default() -> Self {
        Self { 
            time: 0.0, 
            resulotion: [10.0, 10.0], 
            mouse_position: [0.0, 0.0],
            zoom: [0.0, 0.0],
            _pad: 10.0,
        }
    }
}
