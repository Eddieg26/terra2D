pub mod loader;
pub mod renderer;
pub use renderer::SpriteRenderer;

use crate::terra::vertex::Vertex;
use image::EncodableLayout;

#[derive(Clone)]
pub struct Sprite {
    id: u64,
    path: Box<str>,
}

pub struct SpriteData {
    pub vertices: [Vertex; 4],
    pub indices: [u32; 6],
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl Sprite {
    pub fn new(id: u64, path: Box<str>) -> Sprite {
        Sprite { id, path }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn load(&self) -> SpriteData {
        let path: &str = &self.path;

        let image = image::io::Reader::open(path).expect("Failed to load sprite at path: {path}");
        let image = image.decode().expect("Failed to decode image");

        let width = image.width();
        let height = image.height();

        let w_mult: f32 = (image.width() as f32) / (image.width() as f32);
        let h_mult: f32 = (image.height() as f32) / (image.height() as f32);

        let pixels = image.into_rgba8();
        let pixels = pixels.as_bytes().into();

        let indices = [0, 1, 2, 1, 0, 3];

        let vertices = [
            Vertex {
                vertex: [-0.5 * w_mult, 0.5 * h_mult, 0.0, 1.0],
            },
            Vertex {
                vertex: [0.5 * w_mult, -0.5 * h_mult, 1.0, 0.0],
            },
            Vertex {
                vertex: [-0.5 * w_mult, -0.5 * h_mult, 0.0, 0.0],
            },
            Vertex {
                vertex: [0.5 * w_mult, 0.5 * h_mult, 1.0, 1.0],
            },
        ];

        SpriteData {
            vertices,
            indices,
            width,
            height,
            pixels,
        }
    }
}
