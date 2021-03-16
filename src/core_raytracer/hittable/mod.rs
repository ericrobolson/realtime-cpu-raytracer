use super::{
    material::Material,
    ray::Ray,
    vec3::{Point3, Vec3},
};

pub mod sphere;
mod world;

pub use world::*;

pub struct HitRecord {
    pub point: Point3,
    pub normal: Vec3,
    pub t: f32,
    pub front_face: bool,
    pub material: Material,
}

impl HitRecord {
    pub fn new(point: Point3, ray: &Ray, outward_normal: Vec3, t: f32, material: Material) -> Self {
        let front_face = ray.direction().dot(outward_normal) < 0.;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };

        Self {
            point,
            normal,
            front_face,
            t,
            material,
        }
    }
}

pub trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
}
