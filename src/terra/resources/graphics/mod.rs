use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

use vulkano::{buffer::Subbuffer, descriptor_set::PersistentDescriptorSet};

use crate::terra::data::{SpriteData, Vertex};

use super::gpu::GpuResources;

pub struct SpriteResources {
    set: Arc<PersistentDescriptorSet>,
    vertex_buffer: Subbuffer<[Vertex]>,
}

impl SpriteResources {
    pub fn new(
        set: Arc<PersistentDescriptorSet>,
        vertex_buffer: Subbuffer<[Vertex]>,
    ) -> SpriteResources {
        SpriteResources { set, vertex_buffer }
    }

    pub fn set(&self) -> &Arc<PersistentDescriptorSet> {
        &self.set
    }

    pub fn vertex_buffer(&self) -> &Subbuffer<[Vertex]> {
        &self.vertex_buffer
    }
}

pub struct GraphicsResources {
    resources: Rc<RefCell<GpuResources>>,
    sprites: HashMap<u64, SpriteResources>,
}

impl GraphicsResources {
    pub fn new(resources: Rc<RefCell<GpuResources>>) -> GraphicsResources {
        GraphicsResources {
            resources,
            sprites: HashMap::new(),
        }
    }

    pub fn sprite(&self, id: &u64) -> Option<&SpriteResources> {
        self.sprites.get(id)
    }

    pub fn add_sprite(&mut self, data: SpriteData) {
        // TODO: Create Vertex buffer Image, Image View, and Descriptor Set for Sprite
        todo!()
    }

    pub fn remove_sprite(&mut self, id: &u64) {
        self.sprites.remove(id);
    }
}
