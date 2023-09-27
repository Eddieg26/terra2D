use vulkano::buffer::BufferContents;

use crate::terra::data::Color;

#[derive(BufferContents)]
#[repr(C)]
pub struct PerObject {
    pub model: [f32; 16],
    pub color: [f32; 4],
}

impl PerObject {
    pub fn new(model: [f32; 16], color: &Color) -> PerObject {
        let color = color.get();

        PerObject {
            model,
            color: [color[0] as f32, color[1] as f32, color[2] as f32, 1.0],
        }
    }
}
