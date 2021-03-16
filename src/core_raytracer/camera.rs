use super::{
    deg_to_rads,
    ray::Ray,
    vec3::{Point3, Vec3},
};

// Could add focus blur: https://raytracing.github.io/books/RayTracingInOneWeekend.html#defocusblur

pub struct Camera {
    origin: Point3,
    target: Point3,
    up: Vec3,
    lower_left_corner: Point3,
    horizontal: Vec3,
    vertical: Vec3,
    viewport_width: f32,
    viewport_height: f32,
}

impl Camera {
    pub fn new(aspect_ratio: f32, v_fov_deg: f32) -> Self {
        let theta = deg_to_rads(v_fov_deg);
        let h = (theta / 2.).tan();
        let viewport_height = 2. * h;
        let viewport_width = aspect_ratio * viewport_height;

        let focal_length = 1.;

        let origin = Point3::new(0., 0., 0.);
        let horizontal = Vec3::new(viewport_width, 0., 0.);
        let vertical = Vec3::new(0., viewport_height, 0.);
        let lower_left_corner =
            origin - horizontal / 2. - vertical / 2. - Vec3::new(0., 0., focal_length);

        let mut camera = Self {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
            viewport_width,
            viewport_height,
            target: (0., 0., 1.).into(),
            up: Vec3::unit_y(),
        };

        camera.look_at((0., 0., 0.).into(), (0., 0., 1.).into(), Vec3::unit_y());

        camera
    }

    pub fn eye(&self) -> Point3 {
        self.origin
    }

    pub fn target(&self) -> Point3 {
        self.target
    }

    pub fn up(&self) -> Vec3 {
        self.up
    }

    pub fn look_at(&mut self, eye: Point3, target: Point3, up: Vec3) {
        let w = (eye - target).unit_vector();
        let u = up.cross(w).unit_vector();
        let v = w.cross(u);

        self.target = target;
        self.up = up;
        self.origin = eye;

        self.horizontal = self.viewport_width * u;
        self.vertical = self.viewport_height * v;
        self.lower_left_corner = self.origin - self.horizontal / 2. - self.vertical / 2. - w;
    }

    pub fn get_ray(&self, s: f32, t: f32) -> Ray {
        let r = Ray::new(
            self.origin,
            self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin,
        );

        r
    }
}
