use winit::event_loop::EventLoop;

use crate::{
    renderer::renderer::Renderer,
    scene::Scene,
    utils::pool::{Handle, Pool},
};

pub struct Engine {
    pub renderer: Renderer,
    scenes: Pool<Scene>,
    running: bool,
}

impl Engine {
    pub fn new(el: &EventLoop<()>) -> Self {
        Engine {
            renderer: Renderer::new(el),
            scenes: Pool::new(),
            running: true,
        }
    }

    pub fn add_scene(&mut self, scene: Scene) -> Handle<Scene> {
        self.scenes.spawn(scene)
    }

    pub fn borrow_scene(&self, handle: &Handle<Scene>) -> Option<&Scene> {
        if let Some(scene) = self.scenes.borrow(handle) {
            return Some(scene);
        }
        None
    }

    pub fn borrow_scene_mut(&mut self, handle: &Handle<Scene>) -> Option<&mut Scene> {
        if let Some(scene) = self.scenes.borrow_mut(handle) {
            return Some(scene);
        }
        None
    }
    pub fn update(&mut self) {
        let client_size = self.renderer.context.inner_size();
        let aspect_ratio = (client_size.width / client_size.height) as f32;

        for i in 0..self.scenes.capacity() {
            if let Some(scene) = self.scenes.at_mut(i) {
                scene.update(aspect_ratio);
            }
        }
    }

    pub fn render(&mut self) {
        let mut alive_scenes: Vec<&Scene> = Vec::new();
        for i in 0..self.scenes.capacity() {
            if let Some(scene) = self.scenes.at(i) {
                alive_scenes.push(scene);
            }
        }
        self.renderer.render(alive_scenes.as_slice());
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn stop(&mut self) {
        self.running = false;
    }
}
