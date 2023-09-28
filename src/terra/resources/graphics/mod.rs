use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

use vulkano::{
    buffer::{BufferUsage, Subbuffer},
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage},
    format::Format,
    image::{view::ImageView, ImageDimensions, ImmutableImage},
    memory::allocator::MemoryUsage,
};

use crate::terra::{
    data::{SpriteData, Vertex},
    util,
};

use super::gpu::GpuResources;

pub struct SpriteResources {
    image: Arc<ImageView<ImmutableImage>>,
    vertex_buffer: Subbuffer<[Vertex]>,
}

impl SpriteResources {
    pub fn new(
        image: Arc<ImageView<ImmutableImage>>,
        vertex_buffer: Subbuffer<[Vertex]>,
    ) -> SpriteResources {
        SpriteResources {
            image,
            vertex_buffer,
        }
    }

    pub fn image(&self) -> &Arc<ImageView<ImmutableImage>> {
        &self.image
    }

    pub fn vertex_buffer(&self) -> &Subbuffer<[Vertex]> {
        &self.vertex_buffer
    }
}

pub struct GraphicsResources {
    resources: Rc<RefCell<GpuResources>>,
    sprites: HashMap<u64, SpriteResources>,
    sprite_index_buffer: Subbuffer<[u32]>,
}

impl GraphicsResources {
    pub fn new(resources: Rc<RefCell<GpuResources>>) -> GraphicsResources {
        let indices = [0, 1, 2, 1, 0, 3];

        let _resources = resources.borrow();
        let allocator = _resources.memory_alloc();
        let sprite_index_buffer = util::buffer_from_iter(
            allocator,
            indices.into_iter(),
            BufferUsage::INDEX_BUFFER,
            MemoryUsage::Upload,
        );

        GraphicsResources {
            resources: resources.clone(),
            sprites: HashMap::new(),
            sprite_index_buffer,
        }
    }

    pub fn sprite(&self, id: &u64) -> Option<&SpriteResources> {
        self.sprites.get(id)
    }

    pub fn sprite_index_buffer(&self) -> &Subbuffer<[u32]> {
        &self.sprite_index_buffer
    }

    pub fn add_sprite(&mut self, data: SpriteData) {
        // TODO: Create Vertex buffer and Image View
        let resources = self.resources.borrow();
        let allocator = resources.memory_alloc();
        let command_buffer_alloc = resources.command_buffer_alloc();
        let queue = resources.queue();

        let vertex_buffer = util::buffer_from_iter(
            allocator,
            data.vertices.into_iter(),
            BufferUsage::VERTEX_BUFFER,
            MemoryUsage::Upload,
        );

        let mut builder = AutoCommandBufferBuilder::primary(
            command_buffer_alloc,
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let image = util::create_immutable_image(
            allocator,
            command_buffer_alloc,
            queue,
            data.pixels.into_iter(),
            ImageDimensions::Dim2d {
                width: data.width,
                height: data.height,
                array_layers: 1,
            },
            Format::R8G8B8A8_SRGB,
        );

        todo!()
    }

    pub fn remove_sprite(&mut self, id: &u64) {
        self.sprites.remove(id);
    }
}
