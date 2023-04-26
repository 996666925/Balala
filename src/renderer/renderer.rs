use std::num::NonZeroU32;

use glow::{
    Context, HasContext, NativeProgram, NativeShader, NativeUniformLocation, UniformLocation,
};
use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextApi, ContextAttributesBuilder, GlContext, PossiblyCurrentContext, Version},
    display::GetGlDisplay,
    prelude::{GlConfig, GlDisplay, NotCurrentGlContextSurfaceAccessor},
    surface::{GlSurface, Surface, SwapInterval, WindowSurface},
};
use glutin_winit::{DisplayBuilder, GlWindow};
use nalgebra::Vector2;
use once_cell::sync::OnceCell;
use raw_window_handle::HasRawWindowHandle;
use winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use crate::{
    scene::{
        node::{Node, NodeKind},
        Scene,
    },
    utils::pool::Handle,
};

pub static GL: OnceCell<Context> = OnceCell::new();

pub struct GpuProgram {
    id: NativeProgram,
}
impl GpuProgram {
    pub fn create_shader(shader_type: u32, shader_source: &str) -> Result<NativeShader, String> {
        unsafe {
            let gl = GL.get().unwrap();
            let shader = gl.create_shader(shader_type)?;
            gl.shader_source(shader, shader_source);
            gl.compile_shader(shader);
            Ok(shader)
        }
    }

    pub fn from_source(vertex_source: &str, fragment_source: &str) -> Result<GpuProgram, String> {
        unsafe {
            let gl = GL.get().unwrap();

            let vertex_shader = Self::create_shader(glow::VERTEX_SHADER, vertex_source)?;
            let fragment_shader =
                Self::create_shader(glow::FRAGMENT_SHADER, fragment_source).unwrap();
            let program = gl.create_program()?;
            gl.attach_shader(program, vertex_shader);
            gl.delete_shader(vertex_shader);
            gl.attach_shader(program, fragment_shader);
            gl.delete_shader(fragment_shader);
            gl.link_program(program);
   
            Ok(GpuProgram { id: program })
        }
    }
    pub fn get_uniform_location(&mut self, name: &str) -> Option<NativeUniformLocation> {
        unsafe {
            let gl = GL.get().unwrap();
            gl.get_uniform_location(self.id, name)
        }
    }
}

impl Drop for GpuProgram {
    fn drop(&mut self) {
        unsafe {
            let gl = GL.get().unwrap();
            gl.delete_program(self.id);
        }
    }
}

pub struct Renderer {
    pub context: Window,
    pub gl_surface: Surface<WindowSurface>,
    pub gl_context: PossiblyCurrentContext,
    flat_shader: GpuProgram,
    cameras: Vec<Handle<Node>>,
    lights: Vec<Handle<Node>>,
    meshes: Vec<Handle<Node>>,

    /// Scene graph traversal stack
    traversal_stack: Vec<Handle<Node>>,
}

impl Renderer {
    pub fn new(el: &EventLoop<()>) -> Renderer {
        //构建窗口
        let window_builder = WindowBuilder::new()
            .with_title("Balala")
            .with_inner_size(LogicalSize::new(800., 600.))
            .with_resizable(false);

        //构建opnegl context
        let template = ConfigTemplateBuilder::default();
        let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));
        let (mut window, gl_config) = display_builder
            .build(el, template, |configs| {
                configs
                    .reduce(|accum, config| {
                        let transparency_check = config.supports_transparency().unwrap_or(false)
                            && !accum.supports_transparency().unwrap_or(false);
                        if transparency_check || config.num_samples() > accum.num_samples() {
                            config
                        } else {
                            accum
                        }
                    })
                    .unwrap()
            })
            .unwrap();

        let window = window.unwrap();

        let raw_window_handle = window.raw_window_handle();

        let gl_display = gl_config.display();
        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(4, 6))))
            .build(Some(raw_window_handle));

        let not_current_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .unwrap()
        };
        let attrs = window.build_surface_attributes(Default::default());
        let gl_surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };

        let gl_context = not_current_context.make_current(&gl_surface).unwrap();
        gl_surface
            .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
            .expect("设置vsync失败");

        let context = unsafe {
            glow::Context::from_loader_function_cstr(|s| {
                gl_context.display().get_proc_address(s) as *const _
            })
        };

        println!("opengl版本：{:?}", context.version());
        GL.set(context).unwrap();
        let vertex_source = include_str!("./glsl/vertex.glsl");
        let fragment_source = include_str!("./glsl/fragment.glsl");

        Renderer {
            context: window,
            flat_shader: GpuProgram::from_source(&vertex_source, &fragment_source).unwrap(),
            traversal_stack: Vec::new(),
            cameras: Vec::new(),
            lights: Vec::new(),
            meshes: Vec::new(),
            gl_surface,
            gl_context,
        }
    }

    pub fn render(&mut self, scenes: &[&Scene]) {
        let gl = GL.get().unwrap();

        let client_size = self.context.inner_size();

        unsafe {
            gl.clear_color(0.0, 0.63, 0.91, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }

        for scene in scenes.iter() {
            self.meshes.clear();
            self.lights.clear();
            self.cameras.clear();
            self.traversal_stack.clear();
            self.traversal_stack.push(scene.root.clone());
            while !self.traversal_stack.is_empty() {
                if let Some(node_handle) = self.traversal_stack.pop() {
                    if let Some(node) = scene.borrow_node(&node_handle) {
                        match node.borrow_kind() {
                            NodeKind::Mesh(_) => self.meshes.push(node_handle),
                            NodeKind::Light(_) => self.lights.push(node_handle),
                            NodeKind::Camera(_) => self.cameras.push(node_handle),
                            _ => (),
                        }
                        // Queue children for render
                        for child_handle in node.children.iter() {
                            self.traversal_stack.push(child_handle.clone());
                        }
                    }
                }
            }

            unsafe {
                gl.use_program(Some(self.flat_shader.id));
            }
            let u_wvp = self
                .flat_shader
                .get_uniform_location("worldViewProjection")
                .unwrap();

           

            // Render scene from each camera
            for camera_handle in self.cameras.iter() {
                if let Some(camera_node) = scene.borrow_node(&camera_handle) {
                    if let NodeKind::Camera(camera) = camera_node.borrow_kind() {
                        // Setup viewport
                        unsafe {
                            let viewport = camera.get_viewport_pixels(Vector2::new(
                                client_size.width as f32,
                                client_size.height as f32,
                            ));
                            gl.viewport(viewport.x, viewport.y, viewport.width, viewport.height);
                        }

                        let view_projection = camera.get_view_projection_matrix();

                        for mesh_handle in self.meshes.iter() {
                            if let Some(node) = scene.borrow_node(&mesh_handle) {
                                let mvp = view_projection * node.global_transform;
                             

                                unsafe {
                                    gl.use_program(Some(self.flat_shader.id));
                                    gl.uniform_matrix_4_f32_slice(
                                        Some(&u_wvp),
                                        false,
                                        mvp.as_slice(),
                                    );
                                }

                                if let NodeKind::Mesh(mesh) = node.borrow_kind() {
                                    for surface in mesh.surfaces.iter() {
                                        surface.draw();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // self.gl_surface.swap_buffers(&self.gl_context).unwrap();
    }
}
