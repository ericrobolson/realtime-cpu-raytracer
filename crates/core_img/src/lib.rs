use image::{DynamicImage, GenericImage, Rgba};

/// RGBA image. 8bit colors.
#[derive(Clone, Debug)]
pub struct Rgba8Image {
    width: u32,
    height: u32,
    img: image::DynamicImage,
}

impl Rgba8Image {
    /// Creates a new empty RGBA8 image.
    pub fn new(width: u32, height: u32) -> Self {
        let img = image::DynamicImage::new_rgba8(width, height);

        Self { width, height, img }
    }

    pub fn load<P>(path: P) -> Self
    where
        P: AsRef<std::path::Path>,
    {
        use image::io::Reader as ImageReader;

        let img = ImageReader::open(path)
            .unwrap()
            .decode()
            .unwrap()
            .to_rgba8();

        let width = img.width();
        let height = img.height();

        let img = DynamicImage::ImageRgba8(img);

        Self { img, width, height }
    }

    /// Puts a pixel at the given coordinates.
    pub fn put_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
        if x > self.width || y > self.height {
            panic!(
                "Size error! Passed ({:?}, {:?}), size is ({:?}, {:?})!",
                x, y, self.width, self.height
            );
        }

        self.img.put_pixel(x, y, Rgba([r, g, b, a]));
    }

    /// Crops the image.
    pub fn crop(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.img = self.img.crop(x, y, width, height);
        self.width = width;
        self.height = height;
    }

    pub fn flip_y_axis(&mut self) {
        self.img = self.img.flipv();
    }

    /// Returns the raw bytes of the image.
    pub fn bytes(&self) -> &[u8] {
        self.img.as_bytes()
    }

    /// Returns the width of the image.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the image.
    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn save<P>(&self, path: P) -> Result<(), String>
    where
        P: AsRef<std::path::Path>,
    {
        match self.img.save(path) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("{:?}", e)),
        }
    }
}
