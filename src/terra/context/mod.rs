use std::{
    cell::RefCell,
    collections::{hash_map::Iter, HashMap},
    rc::Rc,
};

use crate::{camera::Camera, sprite::SpriteRenderer};

use super::resources::graphics::GraphicsResources;

pub struct GraphicsContext {
    resources: Rc<RefCell<GraphicsResources>>,
    sprite_renderers: HashMap<u64, Vec<Rc<RefCell<SpriteRenderer>>>>,
    camera: Rc<RefCell<Camera>>,
}

impl GraphicsContext {
    pub fn new(resources: &Rc<RefCell<GraphicsResources>>) -> GraphicsContext {
        GraphicsContext {
            sprite_renderers: HashMap::new(),
            resources: resources.clone(),
            camera: Rc::new(RefCell::new(Camera::new())),
        }
    }

    pub fn camera(&self) -> &Rc<RefCell<Camera>> {
        &self.camera
    }

    pub fn sprite_renderers(&self) -> Iter<'_, u64, Vec<Rc<RefCell<SpriteRenderer>>>> {
        self.sprite_renderers.iter()
    }

    pub fn add_sprite_renderer(&mut self, renderer: &Rc<RefCell<SpriteRenderer>>) {
        let instance = renderer.borrow();
        let sprite = instance.sprite();

        if let Some(renderers) = self.sprite_renderers.get_mut(&sprite.id()) {
            renderers.push(renderer.clone());
        } else {
            let mut renderers = vec![];
            renderers.push(renderer.clone());
            self.sprite_renderers.insert(sprite.id(), renderers);
        }

        self.resources.borrow_mut().add_sprite(sprite.load());
    }

    pub fn remove_sprite_renderer(&mut self, renderer: &Rc<RefCell<SpriteRenderer>>) {
        let instance = renderer.borrow();
        let sprite = instance.sprite();

        if let Some(renderers) = self.sprite_renderers.get_mut(&sprite.id()) {
            renderers.retain(|r| r.borrow().id() != instance.id());
        }
    }
}
