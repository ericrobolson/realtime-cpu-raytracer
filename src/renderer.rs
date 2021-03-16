use std::sync::mpsc::{channel, Receiver, Sender};

use core_data_structures::hashmap::HashMap;
use core_img::Rgba8Image;
use core_renderer::{Color, RenderBuilder, RenderCommand, SpriteTransform, TextureId};
use core_text::rasterizer::{CharacterRecord, TextRasterizer};

pub fn build<'a>(
    render_width: u32,
    render_height: u32,
    render_scalar: u32,
    save_renders: bool,
    font: &'static str,
    font_size: u32,
) -> BloodRenderer<'a> {
    BloodRenderer::new(
        render_width,
        render_height,
        render_scalar,
        save_renders,
        font,
        font_size,
    )
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Command {
    pub c: char,
    pub color: Color,
    pub x: u32,
    pub y: u32,
}

impl Command {
    pub fn default() -> Self {
        Self {
            c: '@',
            color: (255, 255, 255).into(),
            x: 0,
            y: 0,
        }
    }

    pub fn reset(&mut self) {
        self.c = '@';
        self.color = (0, 0, 0).into();
    }
}

pub struct BloodRenderer<'a> {
    sender: Sender<Command>,
    receiver: Receiver<Command>,
    commands: Vec<Command>,
    save_renders: bool,
    font: &'static str,
    font_size: u32,
    text_rasterizer: TextRasterizer<'a>,
    next_texture_id: u32,
    render_size: Size,
    characters: HashMap<char, (TextureId, CharacterRecord)>,
}
impl<'a> BloodRenderer<'a> {
    fn new(
        render_width: u32,
        render_height: u32,
        render_scalar: u32,
        save_renders: bool,
        font: &'static str,
        font_size: u32,
    ) -> Self {
        let (sender, receiver) = channel();
        let mut text_rasterizer = TextRasterizer::new(save_renders);

        text_rasterizer.load_font(font_size, font);

        // TODO: preload characters
        let characters = HashMap::new();

        let mut renderer = Self {
            sender,
            receiver,
            render_size: Size {
                width: 0,
                height: 0,
            },
            save_renders,
            commands: vec![],
            font,
            font_size,
            next_texture_id: 0,
            text_rasterizer,
            characters,
        };

        renderer.resize(render_width, render_height, render_scalar);
        renderer
    }

    pub fn resize(&mut self, window_width: u32, window_height: u32, render_scalar: u32) {
        let render_scalar = render_scalar.max(1);

        let render_width = window_width / render_scalar;
        let render_height = window_height / render_scalar;

        self.commands = vec![
            Command {
                x: 0,
                y: 0,
                c: '感',
                color: (0, 0, 0).into(),
            };
            (render_width * render_height) as usize
        ];

        self.render_size = Size {
            width: render_width,
            height: render_height,
        };
    }

    /// The render size
    pub fn size(&self) -> Size {
        self.render_size
    }

    pub fn queue(&self) -> Sender<Command> {
        self.sender.clone()
    }

    pub fn clear(&mut self) {
        for cmd in self.commands.iter_mut() {
            cmd.reset();
        }
    }

    pub fn draw(&mut self, render_pass: &mut impl RenderBuilder) {
        perf!("blood renderer - draw");

        self.sync();

        let mut img = match self.save_renders {
            true => Some(Rgba8Image::new(
                self.render_size.width,
                self.render_size.height,
            )),
            false => None,
        };

        // Render characters
        for cmd in &self.commands {
            // Get character record + texture id
            let (texture_id, character_record) = {
                match self.characters.get(&cmd.c) {
                    Some(rec) => *rec,
                    None => {
                        // Raster character
                        let (record, mut image) =
                            self.text_rasterizer
                                .raster_character(cmd.c, self.font, self.font_size);

                        // Send img to GPU to be loaded
                        let texture_id = TextureId::new(self.next_texture_id);
                        self.next_texture_id = self.next_texture_id.wrapping_add(1);

                        render_pass.queue(RenderCommand::LoadTexture {
                            texture: texture_id,
                            image,
                        });

                        // Store in map
                        let value = (texture_id, record);
                        self.characters.insert(cmd.c, value);

                        // Continue
                        value
                    }
                }
            };

            // Build up render command
            let x = cmd.x as f32 / self.render_size.width as f32;
            let y = cmd.y as f32 / self.render_size.height as f32;
            let width = 1. / self.render_size.width as f32;
            let height = 1. / self.render_size.height as f32;

            render_pass.queue(RenderCommand::DrawSprite {
                texture: texture_id,
                transform: SpriteTransform {
                    x,
                    y, // For some reason, y is flipped and offset. So simply adjust it here.
                    z: 1.,
                    width,
                    height,
                },
                color: cmd.color.into(),
            });

            // Debug rendering
            match img.as_mut() {
                Some(img) => {
                    img.put_pixel(
                        cmd.x,
                        cmd.y,
                        cmd.color.r,
                        cmd.color.g,
                        cmd.color.b,
                        255 as u8,
                    );
                }
                None => {}
            }
        }

        match img {
            Some(img) => {
                img.save("_test/a_test_render.png").unwrap();
            }
            None => {}
        }
    }

    fn sync(&mut self) {
        let width = self.render_size.width;
        let height = self.render_size.height;

        for cmd in self.receiver.try_iter() {
            let x = cmd.x % width;
            let y = cmd.y % height;

            let i = core_conversions::index_2d_to_1d(x as usize, y as usize, width as usize);

            self.commands[i] = cmd;
        }
    }
}
