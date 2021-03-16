// Top level renderer.
// Does things at the scene level, to allow individual backends to handle what's going on.

use core_data_structures::queue::Queue;
use core_img::Rgba8Image;
const RENDER_COMMAND_CAPACITY: usize = 256;

/// Normalized screen coordinates. Everything starts at (0,0) ends at (1,1)
#[derive(Clone, Copy, Debug)]
pub struct SpriteTransform {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub width: f32,
    pub height: f32,
}

/// Sub image of the texture. Starts at top left (0,0) and ends at bottom right (1,1)
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SubImage {
    /// The normalized start x of the sub image (0. - 1.)
    pub start_x: f32,
    /// The normalized start y of the sub image (0. - 1.)
    pub start_y: f32,
    /// The normalized end x of the sub image (0. - 1.)
    pub end_x: f32,
    /// The normalized end y of the sub image (0. - 1.)
    pub end_y: f32,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct TextureId {
    id: u32,
}

impl TextureId {
    pub fn new(id: u32) -> Self {
        Self { id }
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

#[derive(Clone, Debug)]
pub enum RenderCommand {
    /// Drops the texture with the given id. NOTE: TextureIds are not managed by the renderer.
    DropTexture { texture: TextureId },
    /// Loads a texture with the given id. NOTE: TextureIds are not managed by the renderer.
    LoadTexture {
        /// The id of the texture that is to be loaded
        texture: TextureId,
        /// The image to load
        image: Rgba8Image,
    },
    /// Draws the given texture
    DrawSprite {
        texture: TextureId,
        transform: SpriteTransform,
        color: Color,
    },
    /// Draws a sub image of the texture
    DrawSubSprite {
        texture: TextureId,
        sub_image: SubImage,
        transform: SpriteTransform,
        color: Color,
    },
}

impl RenderCommand {
    /// whether the command is guaranteed to execute or not
    pub fn is_guaranteed(&self) -> bool {
        match self {
            RenderCommand::DropTexture { .. } | RenderCommand::LoadTexture { .. } => true,
            _ => false,
        }
    }
}

pub trait RenderBuilder {
    /// Queue up a command for execution
    fn queue(&mut self, command: RenderCommand);
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Into<Color> for (u8, u8, u8, u8) {
    fn into(self) -> Color {
        Color {
            r: self.0,
            g: self.1,
            b: self.2,
            a: self.3,
        }
    }
}

impl Into<Color> for (u8, u8, u8) {
    fn into(self) -> Color {
        Color {
            r: self.0,
            g: self.1,
            b: self.2,
            a: u8::MAX,
        }
    }
}

/// Top level renderer. Functionality based.
pub struct Renderer {
    /// The actual platform specific rendering backend.
    backend: Box<dyn BackendRenderer>,
    dirty: bool,

    guaranteed_commands: Queue<RenderCommand>,
    draw_commands: Queue<RenderCommand>,
}

impl Renderer {
    fn new(backend: Box<dyn BackendRenderer>) -> Self {
        Self {
            dirty: true,
            backend,
            draw_commands: Queue::new(RENDER_COMMAND_CAPACITY),
            guaranteed_commands: Queue::new(RENDER_COMMAND_CAPACITY),
        }
    }

    /// Triggers a new render pass.
    pub fn create_render_pass(&mut self) {
        self.dirty = true;
        self.draw_commands.clear();
    }

    /// Triggers a resize of the window
    pub fn resize(&mut self, w: u32, h: u32) {
        let (w, h) = {
            let mut w = w;
            let mut h = h;
            if w == 0 {
                w = 1;
            }

            if h == 0 {
                h = 1;
            }

            (w, h)
        };

        self.backend.resize(w, h);
    }

    /// Dispatches the render pass.
    pub fn dispatch(&mut self) {
        // Update all draw commands if not dirty
        if self.dirty {
            self.dirty = false;
            self.backend.set_draw_commands(&self.draw_commands.items());
        }

        // Update all required commands
        {
            self.backend
                .set_guaranteed_commands(&self.guaranteed_commands.items());
            self.guaranteed_commands.clear();
        }

        self.backend.dispatch();
    }
}

impl RenderBuilder for Renderer {
    fn queue(&mut self, command: RenderCommand) {
        if command.is_guaranteed() {
            self.guaranteed_commands.push(command);
        } else {
            self.draw_commands.push(command);
        }
    }
}

/// The platform specific backend renderer. Such as OpenGL, Vulkan, WGPU, etc.
pub trait BackendRenderer {
    /// Dispatches all queued draw commands and draws to the screen
    fn dispatch(&mut self);

    /// Resizes the window
    fn resize(&mut self, w: u32, h: u32);

    /// Sets a list of guaranteed commands to be executed on the next pass
    fn set_guaranteed_commands(&mut self, commands: &[RenderCommand]);

    /// Sets a list of draw commands, which may or may not be executed.
    fn set_draw_commands(&mut self, commands: &[RenderCommand]);
}

/// Creates a new renderer
pub fn make_renderer(backend: Box<dyn BackendRenderer>) -> Renderer {
    Renderer::new(backend)
}
