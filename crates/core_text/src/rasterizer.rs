use core_img::Rgba8Image;

use core_fs::load;
use rusttype::{point, Font, Scale, VMetrics};
use std::{borrow::Borrow, collections::HashMap};

/// A simple struct for rastering characters from a font
pub struct TextRasterizer<'a> {
    fonts: HashMap<(&'static str, u32), FontRecord<'a>>,
    debug_save_rasters: bool,
}

struct FontRecord<'a> {
    font: Font<'a>,
    font_size: u32,
    scale: Scale,
    metrics: VMetrics,
    glyph_height: u32,
}

#[derive(Copy, Clone, PartialEq)]
pub struct CharacterRecord {
    pub c: char,
    pub width: u32,
    pub height: u32,
}

impl<'a> TextRasterizer<'a> {
    pub fn new(debug_save_rasters: bool) -> Self {
        Self {
            debug_save_rasters,
            fonts: HashMap::new(),
        }
    }

    pub fn load_font(&mut self, font_size: u32, font: &'static str) {
        let data = core_fs::load(font);
        let font_data = Font::try_from_vec(data).unwrap_or_else(|| {
            panic!(format!("error constructing a Font from data at {:?}", font));
        });

        let scale = Scale::uniform(font_size as f32);
        let metrics = font_data.v_metrics(scale);
        let glyph_height = (metrics.ascent - metrics.descent).ceil() as u32;

        self.fonts.insert(
            (font, font_size),
            FontRecord {
                font: font_data,
                font_size,
                scale,
                metrics,
                glyph_height,
            },
        );
    }

    pub fn raster_character(
        &mut self,
        c: char,
        font: &'static str,
        font_size: u32,
    ) -> (CharacterRecord, Rgba8Image) {
        let font_record = match self.fonts.get(&(font, font_size)) {
            Some(f) => f,
            None => {
                self.load_font(font_size, font);
                self.fonts.get(&(font, font_size)).unwrap()
            }
        };

        let glyph = font_record
            .font
            .glyph(c)
            .scaled(font_record.scale)
            .positioned(point(0.0, 0.0 + font_record.metrics.ascent));

        let height = font_record.glyph_height;

        let mut min_x = u32::MAX;
        let mut max_x = 0;
        let mut min_y = u32::MAX;
        let mut max_y = 0;

        let (img_width, img_height) = (font_size, height);

        let is_whitespace = c.is_whitespace()
            || match glyph.pixel_bounding_box() {
                Some(bb) => false,
                None => true,
            };

        if is_whitespace {
            let character_record = CharacterRecord {
                c,
                width: img_width,
                height: img_height,
            };

            return (character_record, Rgba8Image::new(img_width, img_height));
        }

        let mut img = {
            let mut img = Rgba8Image::new(img_width, img_height);
            let color = (255, 255, 255);

            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                // Draw the glyph into the image per-pixel by using the draw closure
                glyph.draw(|x, y, v| {
                    let px = x + glyph.position().x as u32;
                    let py = y + {
                        // Due to negative offsets, need to ensure that we don't go below 0.
                        let y = bounding_box.min.y;
                        if y < 0 {
                            0
                        } else {
                            bounding_box.min.y as u32
                        }
                    };

                    if v > 0. {
                        if px < min_x {
                            min_x = px;
                        }
                        if px > max_x {
                            max_x = px;
                        }

                        if py < min_y {
                            min_y = py;
                        }
                        if py > max_y {
                            max_y = py;
                        }
                    }
                    img.put_pixel(px, py, color.0, color.1, color.2, output_alpha(v));
                });
            }

            img
        };

        let img_width = max_x - min_x + 1;
        let img_height = img_height;
        img.crop(min_x, 0, img_width, img_height);

        if !c.is_whitespace() && self.debug_save_rasters {
            println!("{:?}", c);
            img.save(format!("_test/_test_img{}.png", c as u32))
                .unwrap();
        }

        let character_record = CharacterRecord {
            c,
            width: img_width,
            height: img_height,
        };

        (character_record, img)
    }
}

fn output_alpha(v: f32) -> u8 {
    let a = v * 255.0;

    return a as u8;
}
