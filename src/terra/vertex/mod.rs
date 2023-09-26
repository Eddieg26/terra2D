use vulkano::buffer::BufferContents;

#[derive(BufferContents, vulkano::pipeline::graphics::vertex_input::Vertex, Clone, Copy, Debug)]
#[repr(C)]
pub struct Vertex {
    #[format(R32G32B32A32_SFLOAT)]
    pub vertex: [f32; 4],
}
