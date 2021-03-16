use crate::renderer::{Command, Size};
use core_time::Timer;

use hittable::{Hittable, World};
use material::Material;
use std::sync::mpsc::{Receiver, Sender};

mod camera;
mod hittable;
mod material;
mod ray;
mod rng;
mod vec3;

use ray::Ray;
use vec3::{Color, Point3, Vec3};

use self::camera::Camera;

const INFINITY: f32 = std::f32::INFINITY;
fn deg_to_rads(degrees: f32) -> f32 {
    degrees * std::f32::consts::PI / 180.
}

const MAX_DRAW: f32 = INFINITY;
const MIN_DRAW: f32 = 0.001;

pub fn build(
    render_size: Size,
    aa_samples: u32,
    post_process_aa: bool,
    primary_ray_strength: u32,
    debug_normals: bool,
) -> Raytracer {
    let max_bounces = 50;

    // Camera
    let aspect_ratio = render_size.width as f32 / render_size.height as f32;
    let v_fov_deg = 90.;
    let mut camera = camera::Camera::new(aspect_ratio, v_fov_deg);
    camera.look_at((-2., 2., 1.).into(), (0., 0., -1.).into(), Vec3::unit_y());

    let eye = camera.eye();
    let target = camera.target();
    let up = camera.up();

    // Build out scene
    // Build out scene
    // TODO: this should not be here
    let world = {
        perf!("raytracer - world gen");

        use hittable::{sphere::*, *};
        let mut world = World::new();

        let material_ground = Material::Lambertian {
            albedo: Color::new(0.8, 0.8, 0.0),
        };

        let s = Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, material_ground);
        world.add(Box::new(s));

        for i in 0..100 {
            let r = rng::random();

            let material_center = Material::Lambertian {
                albedo: (rng::random(), rng::random(), rng::random()).into(),
            };
            let material_left = Material::Dielectric { ior: 1.5 };
            let material_right = Material::Metal {
                albedo: Color::new(rng::random(), rng::random(), rng::random()),
                fuzz: 1.,
            };

            let radius = rng::random_range(0.1, 1.);
            let x = rng::random_range(-10., 10.);
            let y = rng::random_range(0., 1.);
            let z = rng::random_range(-10., 10.);

            if r < 0.20 {
                let s = Sphere::new(Point3::new(x, y, z), radius, material_left);
                world.add(Box::new(s));
            } else if r > 0.2 && r < 0.6 {
                let s = Sphere::new(Point3::new(x, y, z), radius, material_center);
                world.add(Box::new(s));
            } else {
                let s = Sphere::new(Point3::new(x, y, z), radius, material_right);
                world.add(Box::new(s));
            }
        }

        world
    };

    let mut tracer = Raytracer {
        v_fov_deg,
        world,
        aspect_ratio,
        aa_samples,
        max_bounces,
        debug_normals,
        post_process_aa,
        primary_ray_strength,
        camera,
        render_size,

        render_commands: vec![],

        eye,
        target,
        up,
    };

    tracer.resize(render_size);

    tracer
}
pub struct Raytracer {
    // image specs
    aspect_ratio: f32,
    render_size: Size,
    v_fov_deg: f32,

    // world
    world: World,

    /// sampling
    aa_samples: u32,
    max_bounces: u32,
    post_process_aa: bool,
    primary_ray_strength: u32,

    // rendering
    render_commands: Vec<Command>,

    // Camera
    camera: Camera,
    eye: Point3,
    target: Point3,
    up: Vec3,

    // Debugging
    debug_normals: bool,
}

impl Raytracer {
    pub fn resize(&mut self, render_size: Size) {
        let camera = Camera::new(
            render_size.width as f32 / render_size.height as f32,
            self.v_fov_deg,
        );
        let render_commands =
            vec![Command::default(); render_size.width as usize * render_size.height as usize];

        self.camera = camera;
        self.render_commands = render_commands;
        self.render_size = render_size;
    }

    pub fn look_at(
        &mut self,
        eye: (f32, f32, f32),
        target: (f32, f32, f32),
        up: Option<(f32, f32, f32)>,
    ) {
        let eye: Point3 = eye.into();
        let target: Point3 = target.into();
        let up: Vec3 = match up {
            Some(up) => up.into(),
            None => Vec3::unit_y(),
        };

        self.eye = eye;
        self.target = target;
        self.up = up;
    }

    /// Raytraces the scene, sending commands to the renderer.
    /// `render_size` is the number of rays to send
    /// `render_queue` is the mechanism to communicate with the renderer
    pub fn execute_render(&mut self, render_queue: Sender<Command>) {
        perf!("raytracer - execute");

        // Check if camera should be recalculated
        {
            let is_changed = self.eye != self.camera.eye()
                || self.target != self.camera.target()
                || self.up != self.camera.up();

            if is_changed {
                self.camera.look_at(self.eye, self.target, self.up);
            }
        }

        // Render
        let (sender, receiver): (Sender<Command>, Receiver<Command>) = std::sync::mpsc::channel();

        // Queue up commands + do ray tracing
        {
            perf!("raytracer - commands");

            let timer = Timer::new();

            for x in 0..self.render_size.width {
                for y in 0..self.render_size.height {
                    // Get the color from the scene
                    let color = {
                        // Get the initial color for the center of the ray
                        let (u, v) = make_uv(
                            x,
                            y,
                            self.render_size.width,
                            self.render_size.height,
                            0.,
                            0.,
                        );
                        let r = self.camera.get_ray(u, v);
                        let mut color =
                            ray_color(&r, &self.world, self.max_bounces, self.debug_normals);

                        // Do AA
                        for _sample in 0..self.aa_samples {
                            let (u, v) = make_uv(
                                x,
                                y,
                                self.render_size.width,
                                self.render_size.height,
                                rng::random(),
                                rng::random(),
                            );
                            let r = self.camera.get_ray(u, v);
                            color +=
                                ray_color(&r, &self.world, self.max_bounces, self.debug_normals);
                        }

                        color
                    };

                    // Send it off
                    sender
                        .send(Command {
                            c: '感',
                            //c: '█',
                            color: to_color(color, self.aa_samples),
                            x,
                            y,
                        })
                        .unwrap();
                }
            }
        }

        // Receive all commands
        for cmd in receiver.try_iter() {
            let i = core_conversions::index_2d_to_1d(
                cmd.x as usize,
                cmd.y as usize,
                self.render_size.width as usize,
            );

            self.render_commands[i] = cmd;
        }

        // do post processing
        if self.post_process_aa {
            perf!("raytracer - post");

            for y in 0..self.render_size.height as usize {
                for x in 0..self.render_size.width as usize {
                    let i = core_conversions::index_2d_to_1d(x, y, self.render_size.width as usize);

                    let mut cmd = self.render_commands[i];

                    let avg_color = {
                        if !self.post_process_aa {
                            cmd.color
                        } else {
                            let x = cmd.x as usize;
                            let y = cmd.y as usize;

                            let mut x_to_process = vec![];
                            try_add_from_grid(x, self.render_size.width, &mut x_to_process);

                            let mut y_to_process = vec![];
                            try_add_from_grid(y, self.render_size.height, &mut y_to_process);

                            // Set up the default RGB values, scaled to the primary ray
                            let mut r: u32 = cmd.color.r as u32 * self.primary_ray_strength;
                            let mut g: u32 = cmd.color.g as u32 * self.primary_ray_strength;
                            let mut b: u32 = cmd.color.b as u32 * self.primary_ray_strength;

                            for x in &x_to_process {
                                for y in &y_to_process {
                                    let i = core_conversions::index_2d_to_1d(
                                        *x,
                                        *y,
                                        self.render_size.width as usize,
                                    );
                                    let c = self.render_commands[i].color;

                                    r += c.r as u32;
                                    g += c.g as u32;
                                    b += c.b as u32;
                                }
                            }

                            // Determine the number of colors to average, including the primary ray strength
                            let num_colors = x_to_process.len() * y_to_process.len();
                            let num_colors = (num_colors) as u32 + self.primary_ray_strength;

                            let r = r / num_colors;
                            let g = g / num_colors;
                            let b = b / num_colors;

                            (r as u8, g as u8, b as u8).into()
                        }
                    };

                    cmd.color = avg_color;
                }
            }
        }

        for cmd in &self.render_commands {
            render_queue.send(*cmd).unwrap();
        }
    }
}

/// Attempt to add two values to the vec if they're within the bounds
fn try_add_from_grid(val: usize, max: u32, v: &mut Vec<usize>) {
    if val > 0 {
        v.push(val - 1);
    }
    if val < max as usize - 1 {
        v.push(val + 1);
    }
}

fn make_uv(x: u32, y: u32, max_x: u32, max_y: u32, u_offset: f32, v_offset: f32) -> (f32, f32) {
    let u = x as f32 / (max_x - 1) as f32;

    // Flip y, otherwise normals are borked
    let v = ((max_y - 1) - y) as f32 / (max_y - 1) as f32;

    (u, v)
}

fn ray_color(ray: &Ray, world: &World, bounces: u32, debug_normals: bool) -> Color {
    if bounces == 0 {
        return Color::default();
    }

    // Scene stuff
    {
        if let Some(rec) = world.hit(ray, MIN_DRAW, MAX_DRAW) {
            // Bounce a ray to simulate light
            let target = rec.point + Vec3::random_in_hemisphere(rec.normal);

            // Normal test
            if debug_normals {
                return 0.5 * (rec.normal + Color::new(1., 1., 1.));
            }

            if let Some((scattered, attenuation)) = rec.material.scatter(ray, &rec) {
                return attenuation * ray_color(&scattered, world, bounces - 1, debug_normals);
            } else {
                return Color::default();
            }

            // Old
            return 0.5
                * ray_color(
                    &Ray::new(rec.point, target - rec.point),
                    world,
                    bounces - 1,
                    debug_normals,
                );
        }
    }

    // Sky color
    let unit_dir = ray.direction().unit_vector();
    let t = 0.5 * (unit_dir.y + 1.);
    (1. - t) * Color::new(1., 1., 1.) + t * Color::new(0.5, 0.7, 1.)
}

fn to_color(v: Vec3, aa_samples: u32) -> core_renderer::Color {
    let aa_samples = aa_samples as f32 + 1.; // Always add 1 in the case that there's 0 samples

    const C: f32 = 255.;
    let scale = 1. / aa_samples;
    let v = v * scale;
    let v = Vec3::new(v.x.sqrt(), v.y.sqrt(), v.z.sqrt());
    let v = v * C;
    // Divide color by number of samples and gamma correct for gamma = 2.0

    let r: u8 = clamp(v.x, 0., C) as u8;
    let g: u8 = clamp(v.y, 0., C) as u8;
    let b: u8 = clamp(v.z, 0., C) as u8;
    let a: u8 = u8::MAX;

    (r, g, b, a).into()
}

fn clamp(v: f32, min: f32, max: f32) -> f32 {
    if v < min {
        min
    } else if v > max {
        max
    } else {
        v
    }
}
