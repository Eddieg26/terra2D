pub mod camera;
pub mod editor;
pub mod sprite;
pub mod terra;
pub mod transform;

use std::{
    cell::RefCell,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    rc::Rc,
};

use editor::Editor;
use sprite::{loader::SpriteLoader, SpriteRenderer};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};

fn main() {
    let events = EventLoop::new();
    let sprites = SpriteLoader::load();

    // let mut terra = Terra::init(&events);
    // let id = terra.window_id();

    let mut editor = Editor::init(&events);
    let id = editor.window_id();

    if let Some(sprite) = sprites.get(&get_id("./src/assets/sprites/094 - Copy.png")) {
        let gengar = Rc::new(RefCell::new(SpriteRenderer::new(sprite, "Gengar")));
        let mut context = editor.ctx().borrow_mut();
        context.add_sprite_renderer(&gengar);
    }

    events.run(move |event, _, flow| {
        flow.set_wait();

        match event {
            Event::WindowEvent { event, window_id } if window_id == id => {
                editor.update(&event);
                match event {
                    WindowEvent::CloseRequested => flow.set_exit(),
                    WindowEvent::Resized(_) => {
                        // terra.recreate_swapchain();
                        editor.recreate_swapchain()
                    }
                    _ => (),
                }
            }

            Event::RedrawRequested(window_id) if window_id == id => {
                editor.run();
            }

            Event::MainEventsCleared => {
                editor.request_redraw();
            }

            // Event::RedrawEventsCleared => terra.render(),
            _ => (),
        }
    })
}

fn get_id(path: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);

    hasher.finish()
}
