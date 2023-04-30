use std::{path::Path, time::Instant};

use engine::Engine;
use glutin::surface::GlSurface;
use nalgebra::{Matrix4, UnitQuaternion, UnitVector3, Vector2, Vector3};
use scene::{
    node::{Camera, Mesh, Node, NodeKind},
    Scene,
};
use utils::pool::Handle;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window,
};

mod engine;
mod math;
mod renderer;
mod resource;
mod scene;
mod utils;

pub struct Controller {
    move_forward: bool,
    move_backward: bool,
    move_left: bool,
    move_right: bool,
}

pub struct Player {
    camera: Handle<Node>,
    pivot: Handle<Node>,
    controller: Controller,
    yaw: f32,
    pitch: f32,
    last_mouse_pos: Vector2<f32>,
}

impl Player {
    pub fn new(scene: &mut Scene) -> Player {
        let mut camera = Node::new(NodeKind::Camera(Camera::default()));
        camera.set_local_position(Vector3::new(0.0, 2.0, 0.0));

        let mut pivot = Node::new(NodeKind::Base);
        pivot.set_local_position(Vector3::new(0.0, 0.0, 20.0));

        let camera_handle = scene.add_node(camera);
        let pivot_handle = scene.add_node(pivot);
        scene.link_nodes(&camera_handle, &pivot_handle);

        Player {
            camera: camera_handle,
            pivot: pivot_handle,
            controller: Controller {
                move_backward: false,
                move_forward: false,
                move_left: false,
                move_right: false,
            },
            yaw: 0.0,
            pitch: 0.0,
            last_mouse_pos: Vector2::zeros(),
        }
    }

    pub fn update(&mut self, scene: &mut Scene) {
        if let Some(pivot_node) = scene.borrow_node_mut(&self.pivot) {
            let mut velocity = Vector3::<f32>::zeros();
            let look = pivot_node.get_look_vector();
            let side = pivot_node.get_side_vector();

            if self.controller.move_forward {
                velocity += look;
            }
            if self.controller.move_backward {
                velocity -= look;
            }
            if self.controller.move_left {
                velocity += side;
            }
            if self.controller.move_right {
                velocity -= side;
            }

            if let Some(normal) = velocity.try_normalize(0.) {
                pivot_node.offset(normal);
            }
            pivot_node.set_local_rotation(UnitQuaternion::from_axis_angle(
                &Vector3::y_axis(),
                self.yaw.to_radians(),
            ));

            if let Some(camera_node) = scene.borrow_node_mut(&self.camera) {
                camera_node.set_local_rotation(UnitQuaternion::from_axis_angle(
                    &Vector3::x_axis(),
                    self.pitch.to_radians(),
                ));
            }
        }
    }

    pub fn process_event<'a>(&mut self, event: &winit::event::Event<()>) -> bool {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CursorMoved { position, .. } => {
                    let mouse_velocity = Vector2::new(
                        position.x as f32 - self.last_mouse_pos.x,
                        position.y as f32 - self.last_mouse_pos.y,
                    );
                    let sens: f32 = 0.3;

                    self.pitch += mouse_velocity.y * sens;
                    self.yaw -= mouse_velocity.x * sens;

                    if self.pitch > 90.0 {
                        self.pitch = 90.0;
                    } else if self.pitch < -90.0 {
                        self.pitch = -90.0;
                    }

                    self.last_mouse_pos = Vector2::new(position.x as f32, position.y as f32);
                }
                WindowEvent::KeyboardInput { input, .. } => match input.state {
                    ElementState::Pressed => {
                        if let Some(key) = input.virtual_keycode {
                            match key {
                                VirtualKeyCode::W => self.controller.move_forward = true,
                                VirtualKeyCode::S => self.controller.move_backward = true,
                                VirtualKeyCode::A => self.controller.move_left = true,
                                VirtualKeyCode::D => self.controller.move_right = true,
                                _ => (),
                            }
                        }
                    }
                    ElementState::Released => {
                        if let Some(key) = input.virtual_keycode {
                            match key {
                                VirtualKeyCode::W => self.controller.move_forward = false,
                                VirtualKeyCode::S => self.controller.move_backward = false,
                                VirtualKeyCode::A => self.controller.move_left = false,
                                VirtualKeyCode::D => self.controller.move_right = false,
                                _ => (),
                            }
                        }
                    }
                },
                _ => (),
            },
            _ => (),
        }
        false
    }
}

pub struct Level {
    scene: Handle<Scene>,
    player: Player,

    cubes: Vec<Handle<Node>>,
    angle: f32,
}

impl Level {
    pub fn new(engine: &mut Engine) -> Level {
        let mut cubes: Vec<Handle<Node>> = Vec::new();

        let mut scene = Scene::new();

        {
            let mut floor_mesh = Mesh::default();
            floor_mesh.make_cube();
            if let Some(texture) =
                engine.request_texture(Path::new("./src/assets/textures/floor.png"))
            {
                floor_mesh.apply_texture(texture);
            }
            let mut floor_node = Node::new(NodeKind::Mesh(floor_mesh));
            floor_node.set_name("Floor");
            floor_node.set_local_scale(Vector3::new(100.0, 0.1, 100.0));
            scene.add_node(floor_node);
        }

        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    let mut cube_mesh = Mesh::default();
                    cube_mesh.make_cube();
                    if let Some(texture) =
                        engine.request_texture(Path::new("./src/assets/textures/box.png"))
                    {
                        cube_mesh.apply_texture(texture);
                    }
                    let mut cube_node = Node::new(NodeKind::Mesh(cube_mesh));
                    cube_node.set_name("Cube");

                    let pos = Vector3::new(i as f32 * 2.0, j as f32 * 2.0, k as f32 * 2.0);
                    cube_node.set_local_position(pos);
                    cubes.push(scene.add_node(cube_node));
                }
            }
        }

        let player = Player::new(&mut scene);

        Level {
            player,
            cubes,
            angle: 0.0,
            scene: engine.add_scene(scene),
        }
    }

    pub fn update(&mut self, engine: &mut Engine) {
        self.angle += 0.1;

        let rotation = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), self.angle);
        if let Some(scene) = engine.borrow_scene_mut(&self.scene) {
            for node_handle in self.cubes.iter() {
                if let Some(node) = scene.borrow_node_mut(node_handle) {
                    node.set_local_rotation(rotation);
                }
            }

            self.player.update(scene);
        }
    }
}

pub struct Game {
    engine: Engine,
    level: Level,
}

impl Game {
    pub fn new(el: &EventLoop<()>) -> Game {
        let mut engine = Engine::new(el);
        let level = Level::new(&mut engine);
        Game { engine, level }
    }

    pub fn update(&mut self) {
        self.level.update(&mut self.engine);
    }

    pub fn run(mut self, el: EventLoop<()>) {
        let mut last_frame_inst = Instant::now();

        let (mut frame_count, mut accum_time) = (0, 0.0);
        el.run(move |event, _target, control_flow| {
            control_flow.set_poll();

            self.level.player.process_event(&event);
            match event {
                Event::MainEventsCleared => {
                    self.update();
                    self.engine.update();
                    accum_time += last_frame_inst.elapsed().as_secs_f32();
                    last_frame_inst = Instant::now();
                    frame_count += 1;
                    if frame_count == 100 {
                        println!(
                            "Avg frame time {}ms",
                            accum_time * 1000.0 / frame_count as f32
                        );
                        accum_time = 0.0;
                        frame_count = 0;
                    }
                }
                Event::RedrawRequested(_) => {}
                Event::RedrawEventsCleared => {
                    self.engine.render();
                    self.engine
                        .renderer
                        .gl_surface
                        .swap_buffers(&self.engine.renderer.gl_context)
                        .unwrap();
                    self.engine.renderer.context.request_redraw();
                }
                Event::WindowEvent {
                    window_id: _,
                    event,
                } => match event {
                    WindowEvent::CloseRequested => {
                        self.engine.stop();
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => self.engine.stop(),
                    _ => (),
                },
                _ => (),
            }
        });
    }
}

fn main() {
    let el = EventLoop::new();
    Game::new(&el).run(el);
}

#[test]
fn fbx() {
    use fbxcel_dom::any::*;
    use std::fs::File;
    use std::io::BufReader;

    let file = File::open("./src/assets/models/cube.fbx").expect("Failed to open file");
    let reader = BufReader::new(file);

    match AnyDocument::from_seekable_reader(reader).expect("Failed to load document") {
        AnyDocument::V7400(ver, doc) => {
            println!("Loaded FBX DOM successfully: FBX version = {:?}", ver);
            for scene in doc.scenes() {
                println!("Scene object: object_id={:?}", scene.object_id());
                let root_id = scene
                    .root_object_id()
                    .expect("Failed to get root object ID");
                println!("\tRoot object ID: {:?}", root_id);
            }
        }
        _ => panic!("FBX version unsupported by this example"),
    }
}
