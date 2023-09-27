use nalgebra_glm::Mat4;
use vulkano::buffer::BufferContents;

use super::util::mat4_to_array;

#[derive(BufferContents, vulkano::pipeline::graphics::vertex_input::Vertex, Clone, Copy, Debug)]
#[repr(C)]
pub struct Vertex {
    #[format(R32G32B32A32_SFLOAT)]
    pub vertex: [f32; 4],
}

pub struct SpriteData {
    pub vertices: [Vertex; 4],
    pub indices: [u32; 6],
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[derive(BufferContents)]
#[repr(C)]
pub struct GlobalData {
    projection: [f32; 16],
    view: [f32; 16],
}

impl GlobalData {
    pub fn new(projection: Mat4, view: Mat4) -> GlobalData {
        GlobalData {
            projection: mat4_to_array(projection),
            view: mat4_to_array(view),
        }
    }

    pub fn identity() -> GlobalData {
        GlobalData {
            projection: mat4_to_array(Mat4::identity()),
            view: mat4_to_array(Mat4::identity()),
        }
    }
}

pub struct Color([f32; 3]);

impl Color {
    pub fn new() -> Color {
        Color([1.0; 3])
    }

    pub fn set_red(&mut self, value: f32) {
        self.0[0] = value;
    }

    pub fn set_green(&mut self, value: f32) {
        self.0[1] = value;
    }

    pub fn set_blue(&mut self, value: f32) {
        self.0[2] = value;
    }

    pub fn set(&mut self, red: f32, green: f32, blue: f32) {
        self.0[0] = red;
        self.0[1] = green;
        self.0[2] = blue;
    }

    pub fn get(&self) -> &[f32; 3] {
        &self.0
    }
}
