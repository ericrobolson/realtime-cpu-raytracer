use core_data_structures::{hashmap::HashMap, queue::Queue};
use core_img::Rgba8Image;
use core_renderer::{BackendRenderer, Color, RenderCommand, SpriteTransform, SubImage, TextureId};
use glow::*;
use glutin::{ContextWrapper, PossiblyCurrent};

const DRAW_COMMAND_CAPACITY: usize = 256;
const STATE_COMMAND_CAPACITY: usize = 256;

const UNIFORM_SCREEN_SIZE: &'static str = "u_screen_size";

const UNIFORM_VIEW_EYE: &'static str = "u_view_eye";
const UNIFORM_VIEW_TARGET: &'static str = "u_view_target";
const UNIFORM_VIEW_UP: &'static str = "u_view_up";
const UNIFORM_VIEW_MATRIX: &'static str = "u_view_matrix";
const UNIFORM_VIEW_FOV_DEGREES: &'static str = "u_view_fov_degrees";

#[derive(Copy, Clone, Debug)]
struct Texture {
    texture_id: TextureId,
    gl_id: u32,
}

pub fn make(
    w: u32,
    h: u32,
    windowed_context: &ContextWrapper<PossiblyCurrent, glutin::window::Window>,
) -> impl BackendRenderer {
    // TODO: make safe
    let (gl, program) = unsafe {
        // Create context
        let gl = glow::Context::from_loader_function(|s| {
            windowed_context.get_proc_address(s) as *const _
        });

        // Set up gl parameters
        gl.enable(glow::FRAMEBUFFER_SRGB);
        gl.enable(glow::BLEND);
        gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

        //gl.enable(glow::CULL_FACE);
        //gl.enable(glow::DEPTH_TEST);

        let shader_version = "#version 330";

        // Create program + link shaders
        let program = gl.create_program().expect("Cannot create program");

        let vertex_shader_source = std::str::from_utf8(include_bytes!("shader.vert")).unwrap();
        let fragment_shader_source = std::str::from_utf8(include_bytes!("shader.frag")).unwrap();

        let shader_sources = [
            (glow::VERTEX_SHADER, vertex_shader_source),
            (glow::FRAGMENT_SHADER, fragment_shader_source),
        ];

        let mut shaders = Vec::with_capacity(shader_sources.len());

        for (shader_type, shader_source) in shader_sources.iter() {
            let shader = gl
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            gl.shader_source(shader, &format!("{}\n{}", shader_version, shader_source));
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                panic!(gl.get_shader_info_log(shader));
            }
            gl.attach_shader(program, shader);
            shaders.push(shader);
        }

        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!(gl.get_program_info_log(program));
        }

        //cleanup
        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }

        gl.use_program(Some(program)); // Need to call before setting uniforms

        // Update UBOs
        /*
               resize_screen(program, &gl, w, h);
               set_camera(
                   program,
                   &gl,
                   &mut view_state,
                   &core_renderer::Camera::default(),
               );
        */
        // Return
        (gl, program)
    };

    GlowRenderer {
        gl,
        program,
        draw_commands: Vec::with_capacity(DRAW_COMMAND_CAPACITY),
        state_commands: Queue::new(STATE_COMMAND_CAPACITY), //  view_state,
        textures: HashMap::new(),
    }
}

struct GlowRenderer {
    gl: Context,
    program: u32,
    draw_commands: Vec<RenderCommand>,
    state_commands: Queue<RenderCommand>,
    textures: HashMap<TextureId, Texture>,
}

impl GlowRenderer {
    fn execute_command(&mut self, command: &RenderCommand) {
        match command {
            RenderCommand::DropTexture { texture } => {
                self.drop_texture(texture);
            }
            RenderCommand::LoadTexture { texture, image } => {
                let image = image.clone();
                self.create_texture(texture, image);
            }
            RenderCommand::DrawSprite {
                texture,
                transform,
                color,
            } => {
                self.draw_sprite(
                    texture,
                    *transform,
                    SubImage {
                        start_x: 0.,
                        start_y: 0.,
                        end_x: 1.,
                        end_y: 1.,
                    },
                    *color,
                );
            }
            RenderCommand::DrawSubSprite {
                texture,
                sub_image,
                transform,
                color,
            } => {
                self.draw_sprite(texture, *transform, *sub_image, *color);
            }
        }
    }

    fn execute_draw_commands(&mut self) {
        for i in 0..self.draw_commands.len() {
            let cmd = self.draw_commands[i].clone();
            self.execute_command(&cmd);
        }
    }

    fn draw_sprite(
        &self,
        texture: &TextureId,
        transform: SpriteTransform,
        sub_img: SubImage,
        color: Color,
    ) {
        match self.textures.get(&texture) {
            Some(texture) => draw_sprite(&self.gl, texture, transform, sub_img, color),
            None => {}
        }
    }

    fn create_texture(&mut self, texture_id: &TextureId, img: core_img::Rgba8Image) -> Texture {
        // Clear out old texture
        match self.textures.get(&texture_id) {
            Some(tex) => {
                self.drop_texture(texture_id);
            }
            None => {}
        }

        // Create + store texture
        let texture = create_texture(&self.gl, texture_id, img);
        self.textures.insert(*texture_id, texture);

        texture
    }

    fn drop_texture(&mut self, texture_id: &TextureId) {
        todo!()
    }
}

impl BackendRenderer for GlowRenderer {
    fn dispatch(&mut self) {
        unsafe {
            self.gl.use_program(Some(self.program));
            self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }

        self.execute_draw_commands();
    }

    fn set_guaranteed_commands(&mut self, commands: &[RenderCommand]) {
        // right now this is pretty basic, so just go thru and execute all commands
        for command in commands {
            self.execute_command(command);
        }
    }

    fn set_draw_commands(&mut self, commands: &[RenderCommand]) {
        // Clean up the draw commands
        self.draw_commands.clear();

        for command in commands {
            if command.is_guaranteed() {
                self.execute_command(command);
            } else {
                self.draw_commands.push(command.clone());
            }
        }
    }

    fn resize(&mut self, w: u32, h: u32) {
        resize_screen(self.program, &self.gl, w, h);
    }
}

impl Drop for GlowRenderer {
    fn drop(&mut self) {
        unsafe {
            for texture in self.textures.values() {
                self.gl.delete_texture(texture.gl_id);
            }

            self.gl.delete_program(self.program);
        }
    }
}

fn create_texture(gl: &Context, texture_id: &TextureId, mut img: Rgba8Image) -> Texture {
    // Images need to be flipped before displaying
    img.flip_y_axis();

    let mut text = Texture {
        texture_id: *texture_id,
        gl_id: 0,
    };
    unsafe {
        let texture = gl.create_texture().unwrap();
        text.gl_id = texture;
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_S,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_T,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::LINEAR as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::LINEAR as i32,
        );

        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA8 as i32,
            img.width() as i32,
            img.height() as i32,
            0, // always be 0 for legacy
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            Some(img.bytes()),
        );
        gl.generate_mipmap(glow::TEXTURE_2D);
    }

    text
}

fn draw_sprite(
    gl: &Context,
    texture: &Texture,
    transform: SpriteTransform,
    sub_img: SubImage,
    color: Color,
) {
    // NOTE: This could be heavily optimized, but is fine for now.
    //https://learnopengl.com/Getting-started/Textures

    let pos_attributes = 3;
    let color_attributes = 4;
    let tex_coord_attributes = 2;

    let vertice_attributes = pos_attributes + color_attributes + tex_coord_attributes; // 3 positions + 3 colors + 2 color coords
    let vertices: Vec<f32> = {
        // Closure to normalize colors
        let c = |col: u8| -> f32 {
            let max_col = u8::MAX as f32;
            col as f32 / max_col
        };

        let r = c(color.r);
        let g = c(color.g);
        let b = c(color.b);
        let a = c(color.a);

        // Closure to normalize 0. - 1. values to -1. - 1.
        let t = |t: f32| -> f32 { t * 2. - 1. };

        let left_x = t(transform.x);
        let right_x = t(transform.x + transform.width);

        // This maps the 0..1 coordinates to a more 'normal' NDC coordinate
        let y = 1. - transform.y;

        let top_y = t(y);
        let bot_y = t(y - transform.height);

        let z = 0.;

        let vert = |x: f32, y: f32, z: f32, tx: f32, ty: f32| {
            vec![
                //
                x, y, z, // pos
                r, g, b, a, // color
                tx, ty, //tex
            ]
        };

        // Note: SubImage + SpriteTransform has the top left being (0,0) and bottom right as (1,1)
        // OpenGL however treats top left as (0, 1) and bottom right as (1, 0)

        let mut verts = vec![];

        // top right
        verts.append(&mut vert(
            // pos
            right_x,
            top_y,
            z,
            // tex
            sub_img.end_x,
            sub_img.end_y,
        ));
        // bottom right
        verts.append(&mut vert(
            // pos
            right_x,
            bot_y,
            z,
            // tex
            sub_img.end_x,
            sub_img.start_y,
        ));
        // bottom left
        verts.append(&mut vert(
            // pos
            left_x,
            bot_y,
            z,
            // tex
            sub_img.start_x,
            sub_img.start_y,
        ));
        // top left
        verts.append(&mut vert(
            // pos
            left_x,
            top_y,
            z,
            // tex
            sub_img.start_x,
            sub_img.end_y,
        ));

        verts
    };

    let indices: Vec<u32> = vec![
        0, 1, 3, // first triangle
        1, 2, 3, // second triangle
    ];

    let num_verts = vertices.len() / vertice_attributes;
    let num_indices = indices.len();

    unsafe {
        // Gen vao, vbo, ebo
        let vao = gl.create_vertex_array().unwrap();
        let vbo = gl.create_buffer().unwrap();
        let ebo = gl.create_buffer().unwrap();

        // bind vao
        gl.bind_vertex_array(Some(vao));

        // bind + buffer vbo
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            core_conversions::slice_f32_to_u8(&vertices),
            glow::STATIC_DRAW,
        );

        // bind + buffer ebo
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            core_conversions::slice_u32_to_u8(&indices),
            glow::STATIC_DRAW,
        );

        // Do attributes
        {
            let stride = (vertice_attributes * std::mem::size_of::<f32>()) as i32;

            // position
            let index = 0;
            let size = pos_attributes as i32;
            let offset = 0;
            gl.vertex_attrib_pointer_f32(index, size, glow::FLOAT, false, stride, offset);
            gl.enable_vertex_attrib_array(index);

            // color
            let index = index + 1;
            let size = color_attributes as i32;
            let offset = (pos_attributes * std::mem::size_of::<f32>()) as i32;
            gl.vertex_attrib_pointer_f32(index, size, glow::FLOAT, false, stride, offset);
            gl.enable_vertex_attrib_array(index);

            // tex coord
            let index = index + 1;
            let size = tex_coord_attributes as i32;
            let offset = ((color_attributes + pos_attributes) * std::mem::size_of::<f32>()) as i32;
            gl.vertex_attrib_pointer_f32(index, size, glow::FLOAT, false, stride, offset);
            gl.enable_vertex_attrib_array(index);
        }

        // Draw things
        gl.enable(glow::TEXTURE_2D);
        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(texture.gl_id));

        gl.bind_vertex_array(Some(vao));
        gl.draw_elements(glow::TRIANGLES, num_indices as i32, glow::UNSIGNED_INT, 0);

        // cleanup buffers
        gl.delete_vertex_array(vao);
        gl.delete_buffer(vbo);
        gl.delete_buffer(ebo);
    }
}

/// Updates the given uniform
fn uniform<F>(gl: &Context, program: u32, name: &'static str, op: F)
where
    F: Fn(u32) -> (),
{
    unsafe {
        let u = gl.get_uniform_location(program, name);
        match u {
            Some(u) => {
                op(u);
            }
            None => {
                println!(
                    "Unable to find uniform {:?}. Likely it is unbound or unused.",
                    name
                );
            }
        }
    }
}
/*
/// Container for view uniforms
#[derive(Default)]
struct ViewState {
    fov: f32,
    eye: Vec3,
    target: Vec3,
    up: Vec3,
}

fn set_camera(
    program: u32,
    gl: &Context,
    view_state: &mut ViewState,
    camera: &core_renderer::Camera,
) {
    unsafe {
        gl.use_program(Some(program)); // Need to call before setting uniforms

        let mut dirty_view_matrix = false;

        let fov = 45.0;
        if fov != view_state.fov {
            // Update fov
            uniform(gl, program, UNIFORM_VIEW_FOV_DEGREES, |u| {
                gl.uniform_1_f32(Some(&u), fov);
            });
        }

        // Update eye
        if camera.eye != view_state.eye {
            dirty_view_matrix = true;
            view_state.eye = camera.eye;

            let (x, y, z) = camera.eye.into();
            uniform(gl, program, UNIFORM_VIEW_EYE, |u| {
                gl.uniform_3_f32(Some(&u), x, y, z)
            });
        }

        // Update target
        if camera.target != view_state.target {
            dirty_view_matrix = true;
            view_state.target = camera.target;

            let (x, y, z) = camera.target.into();
            uniform(gl, program, UNIFORM_VIEW_TARGET, |u| {
                gl.uniform_3_f32(Some(&u), x, y, z)
            });
        }

        // Update up
        let camera_up = camera.up.unwrap_or(Vec3::unit_y());
        if camera_up != view_state.up {
            dirty_view_matrix = true;
            view_state.up = camera_up;

            let (x, y, z) = camera_up.into();
            uniform(gl, program, UNIFORM_VIEW_UP, |u| {
                gl.uniform_3_f32(Some(&u), x, y, z)
            });
        }

        // Update view matrix
        if dirty_view_matrix {
            uniform(gl, program, UNIFORM_VIEW_MATRIX, |u| {
                gl.uniform_matrix_4_f32_slice(Some(&u), false, camera.to_mat4().as_slice())
            });
        }
    }
}
*/
fn resize_screen(program: u32, gl: &Context, w: u32, h: u32) {
    unsafe {
        gl.use_program(Some(program)); // Need to call before setting uniforms

        // Create screensize ubo
        uniform(gl, program, UNIFORM_SCREEN_SIZE, |u| {
            gl.uniform_2_f32(Some(&u), w as f32, h as f32);
        });

        // Resize viewport
        gl.viewport(0, 0, w as i32, h as i32);
    }
}
