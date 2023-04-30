use std::{cell::RefCell, path::Path, rc::Rc};

use winit::event_loop::EventLoop;

use crate::{
    renderer::renderer::Renderer,
    resource::{texture::Texture, Resource, ResourceKind},
    scene::Scene,
    utils::pool::{Handle, Pool},
};

pub struct Engine {
    pub renderer: Renderer,
    scenes: Pool<Scene>,
    resources: Vec<Rc<RefCell<Resource>>>,
    running: bool,
}

impl Engine {
    pub fn new(el: &EventLoop<()>) -> Self {
        Engine {
            renderer: Renderer::new(el),
            scenes: Pool::new(),
            resources: Vec::new(),
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

    pub fn request_texture(&mut self, path: &Path) -> Option<Rc<RefCell<Resource>>> {
        for existing in self.resources.iter() {
            let resource = existing.borrow_mut();
            if resource.path == path {
                if let ResourceKind::Texture(tex) = resource.borrow_kind() {
                    return Some(existing.clone());
                } else {
                    println!("{:?} 资源不合法!", path);
                    return None;
                }
            }
        }

        if let Ok(texture) = Texture::load(path) {
            let resource = Rc::new(RefCell::new(Resource::new(
                path,
                ResourceKind::Texture(texture),
            )));
            self.resources.push(resource.clone());
            return Some(resource.clone());
        }

        None
    }

    pub fn update(&mut self) {
        let client_size = self.renderer.context.inner_size();
        let aspect_ratio = client_size.width as f32 / client_size.height as f32;
        for i in 0..self.scenes.capacity() {
            if let Some(scene) = self.scenes.at_mut(i) {
                scene.update(aspect_ratio);
            }
        }
    }

    pub fn render(&mut self) {
        self.renderer.upload_resources(&mut self.resources);
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
